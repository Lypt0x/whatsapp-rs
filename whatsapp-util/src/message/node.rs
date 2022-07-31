use serde_json::Value;
use std::collections::HashMap;

#[derive(serde::Serialize, Debug)]
pub struct Node {
    description: String,
    attributes: HashMap<String, Value>,
    content: Value,
}

impl Node {
    pub fn new(description: String, attributes: HashMap<String, Value>, content: Value) -> Self {
        Self {
            description,
            attributes,
            content,
        }
    }

    pub fn from_attributes(description: String, attributes: HashMap<String, Value>) -> Self {
        Self {
            description,
            attributes,
            content: Value::Null,
        }
    }
}
