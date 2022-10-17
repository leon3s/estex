use ntex::web;
use serde_json::json;
use ntex::http::StatusCode;

/// Http Error
#[derive(Debug)]
pub struct HttpError {
  pub(crate) msg: String,
  pub(crate) status: StatusCode,
}

impl std::fmt::Display for HttpError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[{}] {}", self.status, self.msg)
  }
}

impl web::WebResponseError for HttpError {
  // builds the actual response to send back when an error occurs
  fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
    let err_json = json!({ "msg": self.msg });
    web::HttpResponse::build(self.status).json(&err_json)
  }
}
