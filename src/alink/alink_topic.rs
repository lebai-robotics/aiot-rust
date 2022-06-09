use regex::Regex;

pub struct ALinkSubscribeTopic {
    pub topic: &'static str,
    pub reg: Regex,
    pub offset: u8,
}

impl ALinkSubscribeTopic {
    pub fn new(topic: &'static str) -> Self {
        Self {
            topic,
            reg: Regex::new(topic.replacen("+", "(.*)", 3).as_str()).unwrap(),
            offset: 0,
        }
    }

    pub fn new_with_regex(topic: &'static str, reg: Regex) -> Self {
        Self {
            topic,
            reg,
            offset: 0,
        }
    }

    pub fn matches(
        &self,
        topic: &str,
        product_key: &str,
        device_name: &str,
    ) -> Option<Vec<String>> {
        if let Some(caps) = self.reg.captures(topic) {
            if &caps[(self.offset + 1) as usize] == product_key
                && &caps[(self.offset + 2) as usize] == device_name
            {
                return Some(
                    caps.iter()
                        .map(|c| c.map(|x| x.as_str().to_string()).unwrap_or("".to_string()))
                        .collect(),
                );
            }
        }
        None
    }
}
