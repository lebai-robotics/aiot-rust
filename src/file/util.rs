use crate::{Error, Result, ThreeTuple};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref REPLACE: Regex = Regex::new(r"[^a-zA-Z0-9_\.]").unwrap();
}

pub fn filename_for_path(path: impl AsRef<std::path::Path>) -> Result<String> {
    let path = path.as_ref();
    let filename = path.file_name().ok_or(Error::InvalidPath)?;
    let filename = filename.to_string_lossy().to_string();
    let filename = REPLACE.replace_all(&filename, regex::NoExpand("_"));
    let filename = filename.trim_start_matches("_");
    if filename.len() <= 0 {
        return Err(Error::InvalidPath);
    }
    let filename = if filename.len() > 100 {
        filename[..100].to_string()
    } else {
        filename.to_string()
    };
    Ok(filename)
}

#[test]
fn test1() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/test.txt")).unwrap(),
        "test.txt"
    );
}
#[test]
fn test2() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/_bc&^NU,HH.tar.gz")).unwrap(),
        "bc__NU_HH.tar.gz"
    );
}
#[test]
fn test3() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/_你好.txt")).unwrap(),
        ".txt"
    );
}

#[test]
fn test4() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/.env.dev")).unwrap(),
        ".env.dev"
    );
}
