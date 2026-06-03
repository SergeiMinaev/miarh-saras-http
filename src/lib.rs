use std::collections::HashMap;
use std::fmt::Display;
use chrono::{Duration, Utc};
use serde::{Serialize,Deserialize};
use serde_json::Value;


#[derive(Serialize,Deserialize,Debug)]
pub struct Request {
	pub method: String,
	pub host: String,
	pub path: String,
	pub session_id: String,
	pub query: HashMap<String,String>,
	pub body_string: String,
	pub route: HashMap<String, String>,
	pub files: HashMap<String, RequestFile>,
	#[serde(default)]
	pub headers: HashMap<String, String>,
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



static SESSION_LIFETIME_MIN: i64 = 20160;

#[derive(Serialize)]
pub struct Resp {
	pub code: u16,
	pub body: Vec<u8>,
	pub content_type: String,
	pub session_id: Option<String>, // None - no cookie, "" - delete cookie
}

impl Resp {
	pub fn ok(text: &str) -> Resp {
		Self {
			code: 200,
			body: text.to_string().as_bytes().to_vec(),
			content_type: "text/html".to_string(),
			session_id: None,
		}
	}
	pub fn get_resp_bytes(&self) -> Vec<u8> {
		let cookie_line = match &self.session_id {
			None => "".to_string(),
			Some(v) => {
				let mut expires = (
						Utc::now() + Duration::try_minutes(SESSION_LIFETIME_MIN).unwrap()
					).to_rfc2822();
				if v == "" {
					expires = (Utc::now() - Duration::try_days(1).unwrap()).to_rfc2822();
				}
				format!("Set-Cookie: session_id={v}; Secure; HttpOnly; SameSite=Lax; \
					Path=/; \
					Expires={expires}")
			}
		};
		let body: Vec<u8> = self.body.clone();
		let mut r = format!(
			"HTTP/1.1 {}\r\n\
			Content-Length: {}\r\n\
			Content-Type: {}\r\n",
			self.code, body.len(), self.content_type
		);
		if cookie_line != "" {
			r = format!("{r}{cookie_line}\r\n");
		}
		let r = format!("{r}\r\n");
		let mut full_response: Vec<u8> = vec![];
		full_response.extend_from_slice(r.as_bytes());
		full_response.extend_from_slice(&body);
		full_response
	}
	pub fn check_auth(&self) {
	}
	pub fn is_logged(&self) -> bool {
		return false
	}
}

#[derive(Serialize, Debug, Clone)]
pub struct Pagination {
	pub page: u64,
	pub per_page: u64,
	pub total: Option<u64>,
	pub total_pages: Option<u64>,
	pub next_page: Option<u64>,
	pub prev_page: Option<u64>,
}

pub struct JsonResp {
	pub ok: bool,
	pub code: u16,
	pub err: String,
	pub msg: String,
	pub data: Value,
	pub pagination: Option<Pagination>,
	pub session_id: Option<String>,
}
impl JsonResp {
	pub fn ok(msg: &str) -> JsonResp {
		Self {
			ok: true,
			code: 200,
			msg: msg.to_string(),
			err: String::default(),
			data: serde_json::json!({}),
			pagination: None,
			session_id: None,
		}
	}
	pub fn err<E: Display>(msg: &str, err: &E) -> JsonResp {
		Self {
			ok: false,
			code: 400,
			err: err.to_string(),
			msg: msg.to_string(),
			data: serde_json::json!({}),
			pagination: None,
			session_id: None,
		}
	}
	pub fn j_err<E: Display>(msg: &E) -> JsonResp {
		Self {
			ok: false,
			code: 400,
			err: String::default(),
			msg: msg.to_string(),
			data: serde_json::json!({}),
			pagination: None,
			session_id: None,
		}
	}
	pub fn code(&mut self, code: u16) -> &mut JsonResp {
		self.code = code;
		self
	}
	pub fn content<T>(&mut self, content: &T) -> &mut JsonResp where T: Serialize {
		self.data = serde_json::to_value(content).unwrap();
		self
	}
	pub fn string_content(&mut self, content: &str) -> &mut JsonResp {
		self.data = Value::String(content.to_string());
		self
	}
	pub fn session_id(&mut self, session_id: String) -> &mut JsonResp {
		self.session_id = Some(session_id);
		self
	}
	pub fn pagination(&mut self, pagination: Pagination) -> &mut JsonResp {
		self.pagination = Some(pagination);
		self
	}
	pub fn to_http(&mut self) -> Resp {
		let mut obj = serde_json::json!({
			"ok": self.ok,
			"err": self.err,
			"msg": self.msg,
			"data": self.data,
		});
		if let Some(p) = &self.pagination {
			if let serde_json::Value::Object(ref mut map) = obj {
				map.insert("pagination".to_string(), serde_json::to_value(p).unwrap());
			}
		}
		let data = serde_json::to_string(&obj).unwrap();
		if self.session_id.is_some() {
			json_resp_with_session(self.code, data, self.session_id.clone())
		} else {
			json_resp(self.code, data)
		}
	}
}


pub fn text_resp(code: u16, text: String) -> Resp {
	Resp {
		code: code,
		body: text.as_bytes().to_vec(),
		content_type: "text/html".to_string(),
		session_id: None,
	}
}

pub fn json_resp(code: u16, text: String) -> Resp {
	Resp {
		code: code,
		body: text.as_bytes().to_vec(),
		content_type: "application/json".to_string(),
		session_id: None,
	}
}

pub fn json_resp_with_session(code: u16, text: String, session_id: Option<String>) -> Resp {
	Resp {
		code: code,
		body: text.as_bytes().to_vec(),
		content_type: "application/json".to_string(),
		session_id: session_id,
	}
}

pub fn code_resp(code: u16) -> Resp {
	Resp {
		code: code,
		body: "".to_string().as_bytes().to_vec(),
		content_type: "text/html".to_string(),
		session_id: None,
	}
}


pub fn tile_resp(tile: Vec<u8>) -> Resp {
	Resp {
		code: 200,
		body: tile,
		content_type: "application/x-protobuf".to_string(),
		session_id: None,
	}
}


pub fn session_resp(code: u16, session_id: Option<String>) -> Resp {
	Resp {
		code: code,
		body: "{}".to_string().as_bytes().to_vec(), // return empty JSON because apiGet/apiPost wants it
		content_type: "text/html".to_string(),
		session_id: session_id,
	}
}
pub fn del_session_resp() -> Resp {
	session_resp(200, Some("".to_string()))
}

pub fn forbidden() -> Resp {
	text_resp(403, r#"{"ok": false, "msg": "Forbidden"}"#.to_string())
}

pub fn not_found() -> Resp {
	text_resp(404, r#"{"ok": false, "msg": "Not Found"}"#.to_string())
}
