pub use self::error::{Error, Result};
use axum::{
  extract::{Path, Query},
  http::{Method, Uri},
  middleware,
  response::{Html, IntoResponse, Response},
  routing::{get, get_service},
  Json, Router,
};
use ctx::Ctx;
use model::ModelController;
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

mod ctx;
mod error;
mod log;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
  //IInitialize ModelController
  let mc = ModelController::new().await?;

  let routes_apis = web::routes_tickets::routes(mc.clone())
    .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

  let router = Router::new()
    .merge(hello_routes())
    .merge(web::routes_login::routes())
    .nest("/api", routes_apis)
    .layer(middleware::from_fn_with_state(
      mc.clone(),
      web::mw_auth::mw_ctx_resolver,
    ))
    .layer(middleware::map_response(main_response_mapper))
    .layer(CookieManagerLayer::new())
    // axum 不允许存在重复 URL ，如果上面 merge 中没有匹配上路由，那么转到 nest service
    .fallback_service(routes_static());

  // region: Start Server
  let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
  println!("->> LISTENING on{addr}\n");
  let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
  axum::serve(listener, router.into_make_service())
    .await
    .unwrap();
  // endregion: Start Server

  Ok(())
}

async fn main_response_mapper(
  ctx: Option<Ctx>,
  uri: Uri,
  req_method: Method,
  resp: Response,
) -> Response {
  println!("->> {:12} - main_response_mapper", "RESP_MAPPER");
  let uuid = uuid::Uuid::new_v4();
  let service_error = resp.extensions().get::<Error>();
  let client_status_error = service_error.map(|err| err.client_status_and_error());

  //  -- If client error, build the new response.
  let error_response = client_status_error
    .as_ref()
    .map(|(status_code, client_error)| {
      let client_error_body = json!({
        "error": {
          "type": client_error.as_ref(),
          "req_uuid": uuid.to_string()
        }
      });

      println!("  ->> client_error_body: {client_error_body}");

      // Build the new response form the client_error_body
      (*status_code, Json(client_error_body)).into_response()
    });

  // -- Build and log the server log line.
  let client_error = client_status_error.unzip().1;
  log::log_request(uuid, req_method, uri, ctx, service_error, client_error)
    .await
    .unwrap();

  error_response.unwrap_or(resp)
}

fn hello_routes() -> Router {
  Router::new()
    .route("/hello", get(hadler_hello))
    .route("/hello/:name", get(hadler_hello_path))
}

/// 静态服务
fn routes_static() -> Router {
  Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

// region: hello controller
#[derive(Debug, Deserialize)]
struct HelloParams {
  name: Option<String>,
}

async fn hadler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
  println!("->> {:<12} - handler_hello - params {params:?}", "HANDLER");
  // get the name from params if available, else use 'world' as default word!
  let name = params.name.as_deref().unwrap_or("world!");
  Html(format!("<div>hello {name}!</div>"))
}

async fn hadler_hello_path(Path(name): Path<String>) -> impl IntoResponse {
  println!("->> {:<12} - handler_hello - params {name:?}", "HANDLER");
  Html(format!("<div>hello {name}!</div>"))
}
// endregion: hello controller
