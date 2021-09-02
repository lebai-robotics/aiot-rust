use reqwest::{Client, Method, Body, Response};
use std::fs;
use std::sync::Arc;
use std::os::windows::fs::FileExt;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Receiver;
use reqwest::header::ToStrError;
use std::num::ParseIntError;
use std::fs::{DirEntry, OpenOptions, File};
use tokio::sync::mpsc::error::SendError;
use log::*;
use std::io::{Write, Seek};

#[derive(Debug)]
pub struct DownloadProcess {
	pub percent: f64,
	pub size: u64,
	pub current: u64,
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	DownloadProcessSendError(#[from] SendError<DownloadProcess>),
	#[error(transparent)]
	IoError(#[from] std::io::Error),
	#[error(transparent)]
	UrlParseError(#[from] url::ParseError),
	#[error(transparent)]
	RequestError(#[from] reqwest::Error),
	#[error(transparent)]
	HttpError(#[from] crate::http::HttpError),
	#[error(transparent)]
	ToStrError(#[from] ToStrError),
	#[error(transparent)]
	ParseIntError(#[from] ParseIntError),
	#[error("数据为空")]
	EmptyData,
	#[error("下载出错，数据不一致")]
	InconsistentData,
	#[error("文件路径错误")]
	FilePathError,
}


pub struct HttpDownloadConfig {
	pub block_size: u64,
	pub uri: String,
	pub file_path: String,
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct HttpDownloader {
	config: HttpDownloadConfig,
	client: Client,
	process_sender: mpsc::Sender<DownloadProcess>,
	process_receiver: Arc<Mutex<mpsc::Receiver<DownloadProcess>>>,
}

impl HttpDownloader {
	async fn download_block(&self, start: u64, end: u64) -> Result<Response> {
		let request = self.client.request(Method::GET, &self.config.uri)
			.header("range", format!("bytes={}-{}", start, end))
			.build()?;
		let response = self.client.execute(request).await?;
		Ok(response)
	}
	async fn download_block_and_write(&self, size: u64, index: u64, start: u64, end: u64) -> Result<()> {
		debug!("download_block_and_write start {} start:{},end:{}", index + 1, start, end);
		let response = self.download_block(start, end).await?;
		let file_name = format!("{}{}", &self.config.file_path, index);

		let bytes = response.bytes().await?;
		fs::write(&file_name, bytes.iter())?;
		debug!("write {}", file_name);

		self.process_sender.send(DownloadProcess {
			percent: (end / size) as f64,
			size,
			current: end,
		}).await?;

		debug!("download_block_and_write end {} start:{},end:{}", index + 1, start, end);
		Ok(())
	}

	async fn download(&self) -> Result<fs::File> {
		let request = self.client.get(&self.config.uri).build()?;
		let response = self.client.execute(request).await?;
		let file_name = &self.config.file_path;

		let bytes = response.bytes().await?;
		let bytes = bytes.to_vec();

		let mut result_file = OpenOptions::new()
			.create(true)
			.write(true)
			.open(file_name)?;
		result_file.write_all(&bytes);
		result_file.flush();
		debug!("write {}", file_name);

		let size = bytes.len() as u64;
		self.process_sender.send(DownloadProcess {
			percent: 1f64,
			size,
			current: size,
		}).await?;
		Ok(result_file)
	}

	/// 写入文件
	async fn write_file(&self, response: Response, file_name: &str) -> Result<()> {
		let bytes = response.bytes().await?;
		fs::write(file_name, bytes.iter())?;
		debug!("write {}", file_name);
		Ok(())
	}

	/// 合并文件
	fn merge(&self, size: u64) -> Result<fs::File> {
		let file_path = std::path::Path::new(&self.config.file_path);
		let dirs = fs::read_dir(&file_path.parent().ok_or(Error::FilePathError)?)?;

		let file_name = &self.config.file_path;
		let mut result_file = OpenOptions::new()
			.create(true)
			.write(true)
			.open(file_path)?;
		let count = dirs.filter(|n| n.is_ok())
			.map(|n| n.unwrap())
			.filter(|n| n.path().to_str().unwrap().starts_with(file_name))
			.count();
		let mut result_size = 0;
		for i in 0..count {
			let path = format!("{}{}", file_name, i);
			let bytes = &fs::read(&path)?;
			result_size = result_size + bytes.len();
			result_file.write(bytes);
			// 删除临时文件
			fs::remove_file(&path)?;
			debug!("remove {}", path)
		}
		result_file.flush()?;
		if size as usize == result_size {
			debug!("合并文件完成");
		} else {
			return Err(Error::InconsistentData);
		}
		Ok(result_file)
	}

	pub fn new(config: HttpDownloadConfig) -> Self {
		let (tx, rx) = mpsc::channel::<DownloadProcess>(10);
		Self {
			config,
			client: Client::new(),
			process_receiver: Arc::new(Mutex::new(rx)),
			process_sender: tx,
		}
	}

	pub fn get_process_receiver(&self) -> Arc<Mutex<Receiver<DownloadProcess>>> {
		self.process_receiver.clone()
	}

	// pub async fn run(self: Arc<Self>) -> Result<()> {
	pub async fn start(&self) -> Result<fs::File> {
		let request = self.client.request(Method::HEAD, &self.config.uri).build()?;
		let response = self.client.execute(request).await?;
		let headers = response.headers();
		let accept_ranges = headers.get("accept-ranges");
		let content_length = headers.get("content-length");
		let accept_ranges_flag = match accept_ranges {
			None => false,
			Some(v) => v.to_str()?.eq("bytes")
		};
		if accept_ranges_flag && content_length.is_some() {
			debug!("支持并发下载");
			let size = content_length.unwrap().to_str()?.parse::<u64>()?;
			if size == 0 {
				debug!("数据为空");
				return Err(Error::EmptyData);
			}
			let t_size = size / self.config.block_size;
			if t_size <= 1 {
				debug!("数据分片 <= 1，单线程下载");
				return Ok(self.download().await?);
			}
			let first_attach = size % self.config.block_size;
			debug!("数据块长度 {}", size);
			debug!("启用 {} 个线程下载", t_size);

			let mut futures = vec![
				Box::pin(self.download_block_and_write(size, 0, 0, self.config.block_size - 1 + first_attach))
			];
			for i in 1..t_size {
				let start = i * self.config.block_size + first_attach;

				let t = self.download_block_and_write(size, i, start, start + self.config.block_size - 1);
				futures.push(Box::pin(t))
			}
			let results = futures_util::future::join_all(futures.into_iter()).await;
			for result in results {
				result?
			}
			debug!("下载完成，开始合并文件");
			Ok(self.merge(size)?)
		} else {
			debug!("不支持并发下载");
			Ok(self.download().await?)
		}
	}
}


