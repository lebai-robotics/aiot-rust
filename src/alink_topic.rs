use serde::Serialize;
use regex::Regex;
use crate::{ThreeTuple, Error, Executor};
use std::marker::PhantomData;
use std::any::TypeId;

/*pub trait ALinkTopicFilter {
	fn filter(&self, topic: &str, executor: Box<dyn Executor>) -> bool;
}*/

/*pub struct ALinkDeviceFilter {
	three: ThreeTuple,

}

impl ALinkTopicFilter for ALinkDeviceFilter {
	fn filter(&self, topic: &str, executor: Box<dyn Executor>) -> bool {
		if &caps[1] != executor.three.product_key || &caps[2] != executor.three.device_name {
			return Ok(());
		}
	}
}*/

pub struct ALinkSubscribeTopic {
	pub topic: &'static str,
	pub topic_regex: Regex,
	pub payload_type_id: TypeId,
	offset: u8,
}

impl ALinkSubscribeTopic {
	pub fn new(topic: &'static str, payload_type_id: TypeId) -> Self {
		Self {
			topic,
			topic_regex: Regex::new(topic.replace("/+/", "/(.*)/").as_str()).unwrap(),
			payload_type_id,
			offset: 0,
		}
	}

	pub fn new_with_regex(topic: &'static str, payload_type_id: TypeId, topic_regex: Regex) -> Self {
		Self {
			topic,
			topic_regex,
			payload_type_id,
			offset: 0,
		}
	}

	pub fn is_match(&self, topic: &str, product_key: &str, device_name: &str) -> bool {
		if let Some(caps) = self.topic_regex.captures(topic) {
			if &caps[(self.offset + 1) as usize] == product_key && &caps[(self.offset + 2) as usize] == device_name {
				return true;
			}
		}
		false
	}
}