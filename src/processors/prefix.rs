use crate::executor::Processor;
use async_trait::async_trait;

pub struct Prefix {
    prefix: String,
}

impl Prefix {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

#[async_trait]
impl Processor for Prefix {
    fn process(&mut self, input: String) -> anyhow::Result<String> {
        Ok(format!("{} {}", self.prefix, input))
    }
}
