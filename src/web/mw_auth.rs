use async_trait::async_trait;
use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

use crate::ctx::Ctx;
use crate::web::{self, AUTH_TOKEN};
use crate::{Error, Result};

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request, next: Next) -> Result<Response> {
  println!("->> {:<12} - mw_request_auth", "MIDDLEWARE");

  println!("->> {:<12} - mw_request_auth user {ctx:?}", "MIDDLEWARE",);

  ctx?;

  Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(cookies: Cookies, mut req: Request, next: Next) -> Result<Response> {
  println!("->> {:12} - mw_ctx_resolver", "MIDDLEWARE");

  let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

  let result_ctx = match auth_token
    .ok_or(Error::AuthFailNoAuthTokenCookie)
    .and_then(parse_token)
  {
    Ok((user_id, _exp, _sign)) => Ok(Ctx::new(user_id)),
    Err(e) => Err(e),
  };

  // Remove the cookie if something went wrong other than NoAuthTokenCookie.
  if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
    cookies.remove(Cookie::from(web::AUTH_TOKEN))
  }

  // Store the ctx_result in the request extension.
  req.extensions_mut().insert(result_ctx);

  Ok(next.run(req).await)
}

// Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
  type Rejection = Error;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
    println!("->> {:12} - Ctx", "EXTRATOR");
    parts
      .extensions
      .get::<Result<Ctx>>()
      .ok_or(Error::AuthFailCtxNotInRequestExt)?
      .clone()
    // let cookies = parts.extract::<Cookies>().await.unwrap();

    // let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // let (user_id, _exp, _sign) = auth_token
    //   .ok_or(Error::AuthFailNoAuthTokenCookie)
    //   .and_then(parse_token)?;

    // Ok(Ctx::new(user_id))
  }
}

/// Parse a token of format 'user-[user-id].[expiratio].[signature]'
/// Returns (user_id, expiration, signathre)
fn parse_token(token: String) -> Result<(u64, String, String)> {
  let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token,)
    .ok_or(Error::AuthFailTokenWrongFormat)?;

  let user_id: u64 = user_id
    .parse()
    .map_err(|_| Error::AuthFailTokenWrongFormat)?;

  Ok((user_id, exp.to_string(), sign.to_string()))
}
