use regex::Regex;

pub struct ALinkSubscribeTopic {
    pub topic: &'static str,
    pub topic_regex: Regex,
    offset: u8,
}

impl ALinkSubscribeTopic {
    pub fn new(topic: &'static str) -> Self {
        Self {
            topic,
            topic_regex: Regex::new(topic.replace("/+/+/", "/(.+?)/(.+?)/").as_str()).unwrap(),
            offset: 0,
        }
    }
    pub fn new_we(topic: &'static str) -> Self {
        Self {
            topic,
            topic_regex: Regex::new(topic.replace("/+/+", "/(.+?)/(.+?)").as_str()).unwrap(),
            offset: 0,
        }
    }

    pub fn new_with_regex(topic: &'static str, topic_regex: Regex) -> Self {
        Self {
            topic,
            topic_regex,
            offset: 0,
        }
    }

    pub fn is_match(&self, topic: &str, product_key: &str, device_name: &str) -> bool {
        if let Some(caps) = self.topic_regex.captures(topic) {
            if &caps[(self.offset + 1) as usize] == product_key
                && &caps[(self.offset + 2) as usize] == device_name
            {
                return true;
            }
        }
        false
    }
}
