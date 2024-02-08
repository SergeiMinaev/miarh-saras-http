use std::collections::HashMap;
use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize)]
pub struct Request {
    pub method: String,
    pub host: String,
    pub path: String,
    pub session_id: String,
    pub query: HashMap<String,String>,
    pub body_string: String,
    pub route: HashMap<String, String>,
    pub files: HashMap<String, RequestFile>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RequestFile {
  pub name: String,
  pub content: Vec<u8>,
}

impl Request {
    pub fn make_struct<T: for<'de> serde::Deserialize<'de> + Clone>(&self) -> Result<T, serde_json::Error> {
      serde_json::from_str::<T>(&self.body_string.clone())
    }
}
