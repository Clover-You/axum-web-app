use anyhow::{Ok, Result};
use serde_json::json;

const BASE_URL: &str = "http://127.0.0.1:8000";

#[tokio::test]
async fn quick_dev() -> Result<()> {
  let request = httpc_test::new_client(BASE_URL)?;
  request.do_get("/hello/clover").await?.print().await?;
  request.do_get("/src/main.rs").await?.print().await?;

  let req_login = request.do_post(
    "/api/login",
    json!({
      "username": "demo1",
      "pwd": "welcome"
    }),
  );
  req_login.await?.print().await?;

  let req_create_ticket = request.do_post(
    "/api/tickets",
    json!({
      "title": "Ticket AAA"
    }),
  );
  req_create_ticket.await?.print().await?;

  request.do_delete("/api/tickets/1").await?.print().await?;

  request.do_get("/api/tickets").await?.print().await?;

  Ok(())
}
