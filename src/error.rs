use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, strum_macros::AsRefStr, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Error {
  LoginFail,

  AuthFailTokenWrongFormat,
  AuthFailNoAuthTokenCookie,
  AuthFailCtxNotInRequestExt,

  // Model errors
  TicketDeleteFailIdNotFound { id: u64 },
}

impl IntoResponse for Error {
  fn into_response(self) -> axum::response::Response {
    println!("->> {:12} - {self:?}", "INFO_RES");
    // (StatusCode::INTERNAL_SERVER_ERROR, "UNHADLED_CLIENT_ERROR").into_response()
    let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
    // self.client_status_and_error()
    response.extensions_mut().insert(self);

    response
  }
}

impl Error {
  pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
    // <-- 最下面的 _ => xx 是永远无法执行的，因为 match 中已经把所有的枚举进行处理，此时会发生一个警告，这个特征允许存在永远无法执行的代码
    #[allow(unreachable_patterns)]
    match self {
      Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

      // -- Auth
      Self::AuthFailCtxNotInRequestExt
      | Self::AuthFailNoAuthTokenCookie
      | Self::AuthFailTokenWrongFormat => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

      // -- Model
      Self::TicketDeleteFailIdNotFound { .. } => {
        (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
      }

      _ => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ClientError::SERVICE_ERROR,
      ),
    }
  }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
  LOGIN_FAIL,
  NO_AUTH,
  INVALID_PARAMS,
  SERVICE_ERROR,
}
