use serde_json::Value;
use std::collections::HashMap;

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

    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    pub fn attributes_clone(&self) -> HashMap<String, Value> {
        self.attributes.clone()
    }

    pub fn size(&self) -> usize {
        2 * self.attributes.len() + !self.content.is_null() as usize + 1
    }

    pub fn id(&self) -> Option<&str> {
        self.attributes.get("id").and_then(Value::as_str)
    }

    pub fn content<'a, T>(&'a self) -> Option<T>
    where
        &'a str: TryInto<T>
    {
        self.content.as_str()?.try_into().ok()
    }

    pub fn children(&self) -> Vec<&Value> {
        self.into_iter().collect()
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

    pub fn deserialize(node: Value) -> Option<Self> {
        serde_json::from_value(node).ok()
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