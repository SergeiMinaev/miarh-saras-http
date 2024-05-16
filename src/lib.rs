use std::collections::HashMap;
use chrono::{Duration, Utc};
use serde_json::{ json, Value };
use serde::{Serialize,Deserialize};


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



static SESSION_LIFETIME_MIN: i64 = 300;

pub struct Resp {
    pub code: u16,
    pub text: String,
    pub content_type: String,
    pub session_id: Option<String>, // None - no cookie, "" - delete cookie
}

impl Resp {
	pub fn ok(text: &str) -> Resp {
		Self {
			code: 200,
			text: text.to_string(),
			content_type: "text/html".to_string(),
			session_id: None,
		}
	}
    pub fn get_resp(&self) -> String {
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
        let mut r = format!(
            "HTTP/1.1 {}\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n",
            self.code, self.text.len(), self.content_type
        );
        if cookie_line != "" {
            r = format!("{r}{cookie_line}\r\n");
        }
        format!("{r}\r\n{}", self.text)
    }
    pub fn check_auth(&self) {
    }
    pub fn is_logged(&self) -> bool {
        return false
    }
}

pub struct JsonResp {
    pub ok: bool,
    pub code: u16,
    pub msg: String,
    pub data: Value,
    pub session_id: Option<String>,
}
impl JsonResp {
    pub fn ok(msg: &str) -> JsonResp {
        Self {
            ok: true,
            code: 200,
            msg: msg.to_string(),
            data: json!({}),
            session_id: None,
        }
    }
    pub fn err(msg: &str) -> JsonResp {
        Self {
            ok: false,
            code: 400,
            msg: msg.to_string(),
            data: json!({}),
            session_id: None,
        }
    }
    pub fn code(&mut self, code: u16) -> &mut JsonResp {
        self.code = code;
        self
    }
    pub fn content(&mut self, content: Value) -> &mut JsonResp {
        self.data = content;
        self
    }
    pub fn session_id(&mut self, session_id: String) -> &mut JsonResp {
        self.session_id = Some(session_id);
        self
    }
    pub fn to_http(&mut self) -> Resp {
        let data = json!({"ok": self.ok, "msg": self.msg, "data": self.data});
        if self.session_id.is_some() {
          json_resp_with_session(self.code, data.to_string(), self.session_id.clone())
        } else {
          json_resp(self.code, data.to_string())
        }
    }
}


pub fn text_resp(code: u16, text: String) -> Resp {
    Resp {
        code: code,
        text: text,
        content_type: "text/html".to_string(),
        session_id: None,
    }
}

pub fn json_resp(code: u16, text: String) -> Resp {
    Resp {
        code: code,
        text: text,
        content_type: "application/json".to_string(),
        session_id: None,
    }
}

pub fn json_resp_with_session(code: u16, text: String, session_id: Option<String>) -> Resp {
    Resp {
        code: code,
        text: text,
        content_type: "application/json".to_string(),
        session_id: session_id,
    }
}

pub fn code_resp(code: u16) -> Resp {
    Resp {
        code: code,
        text: "".to_string(),
        content_type: "text/html".to_string(),
        session_id: None,
    }
}

pub fn session_resp(code: u16, session_id: Option<String>) -> Resp {
    Resp {
        code: code,
        text: "{}".to_string(), // return empty JSON because apiGet/apiPost wants it
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
    text_resp(404, "Not Found".to_string())
}
