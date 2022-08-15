pub use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use crate::model::ContactJid;

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Node {
    description: String,
    attributes: HashMap<String, Value>,
    content: Value,
}

pub struct NodeContentIterator<'a> {
    inclusive: bool,
    content: &'a Value
}

pub trait DataExt : Debug {
    fn description(&self) -> &str;
    fn size(&self) -> usize;
    fn id(&self) -> Option<&str>;
    fn content<'a, T>(&'a self) -> Option<T>
        where
            &'a str: TryInto<T>;

    fn has_content(&self) -> bool;

    fn content_as_value(&self) -> &Value;

    fn content_array_nums(&self) -> Option<Vec<u8>>;

    fn attribute(&self, key: &str) -> Option<&Value>;

    fn error_code(&self) -> Option<u32> {
        if self.id()? == "stream:error" {
            return self.attribute("code")?.as_str()?.parse::<u32>().ok();
        }

        None
    }
}

impl Node {
    pub fn new(
        description: String,
        attributes: HashMap<String, Value>,
        content: Value
    ) -> Self {
        Self {
            description,
            attributes,
            content,
        }
    }

    pub fn from_attributes(
        description: String,
        attributes: HashMap<String, Value>
    ) -> Self {
        Self {
            description,
            attributes,
            content: Value::Null,
        }
    }
    
    pub fn children(&self) -> Vec<&Value> {
        self.into_iter().collect()
    }

    pub fn attributes_clone(&self) -> HashMap<String, Value> {
        self.attributes.clone()
    }

    pub fn find_description(&self, description: &str) -> Option<&Value> {
        for item in self.into_iter() {
            match item {
                Value::Array(values) => {
                    return values.into_iter().find(|node| node["description"].as_str() == description.into())
                },

                Value::String(content) => {
                    if content == description {
                        return item.into()
                    }
                },

                _ => ()
            }
        }

        None
    }

    pub fn parse_jid(jid: &Value) -> ContactJid {
        let user = jid["user"].as_str().unwrap().to_string();
        let device = jid["device"].as_u64().unwrap() as u32;
        let agent = jid["agent"].as_u64().unwrap() as u32;

        ContactJid::from_companion(user, device, agent)
    }

    pub fn deserialize(node: Value) -> Option<Self> {
        serde_json::from_value(node).ok()
    }

    pub fn serialize(node: Node) -> Option<Value> { serde_json::to_value(node).ok() }

}

impl DataExt for Node {
    fn description(&self) -> &str {
        self.description.as_str()
    }

    fn size(&self) -> usize {
        2 * self.attributes.len() + !self.content.is_null() as usize + 1
    }

    fn id(&self) -> Option<&str> {
        self.description.as_str().into()
    }

    fn content<'a, T>(&'a self) -> Option<T>
        where
            &'a str: TryInto<T>
    {
        self.content.as_str()?.try_into().ok()
    }

    fn has_content(&self) -> bool {
        !self.content.is_null()
    }

    fn content_as_value(&self) -> &Value {
        &self.content
    }

    fn content_array_nums(&self) -> Option<Vec<u8>> {
        match &self.content {
            Value::Array(values) => {
                Some(values.into_iter().map(|value| value.as_u64().unwrap() as u8).collect())
            },
            _ => None
        }
    }

    fn attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }
}

impl DataExt for Value {
    fn description(&self) -> &str {
        self["description"].as_str().unwrap_or_default()
    }

    fn size(&self) -> usize {
        2 * self["attributes"].as_object().unwrap().len()
            + !self["content"].is_null() as usize + 1
    }

    fn id(&self) -> Option<&str> {
        self["attributes"].as_object().unwrap()["id"].as_str()
    }

    fn content<'a, T>(&'a self) -> Option<T> where &'a str: TryInto<T> {
        self["content"].as_str()?.try_into().ok()
    }

    fn has_content(&self) -> bool {
        !self["content"].is_null()
    }

    fn content_as_value(&self) -> &Value {
        &self["content"]
    }

    fn content_array_nums(&self) -> Option<Vec<u8>> {
        self["content"].as_array().map(|val| {
            val.into_iter().map(|v| v.as_u64().unwrap() as u8).collect()
        })
    }

    fn attribute(&self, key: &str) -> Option<&Value> {
        self["attributes"].get(key)
    }
}

impl TryFrom<Value> for Node {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(value)?)
    }
}

impl<'a> TryInto<Node> for &'a Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Node, Self::Error> {
        Ok(serde_json::from_value(self.clone())?)
    }
}

impl<'a> IntoIterator for &'a Node {
    type Item = &'a Value;
    type IntoIter = NodeContentIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            inclusive: true,
            content: &self.content
        }
    }
}

impl<'a> Iterator for NodeContentIterator<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inclusive {
            self.inclusive = false;
            return self.content.into()
        }

        self.content.get("content")
    }
}