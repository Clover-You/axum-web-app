use crate::ctx::Ctx;
use crate::model::{ModelController, Ticket, TicketForCreate};
use crate::Result;
use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Json, Router};

async fn create_ticket(
  ctx: Ctx,
  State(mc): State<ModelController>,
  Json(ticket_fc): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
  let ticket = mc.create_ticket(ctx, ticket_fc).await?;
  Ok(Json(ticket))
}

async fn list_tickets(ctx: Ctx, State(mc): State<ModelController>) -> Result<Json<Vec<Ticket>>> {
  let tickets = mc.list_tickets(ctx).await?;
  Ok(Json(tickets))
}

async fn delete_ticket(
  ctx: Ctx,
  State(mc): State<ModelController>,
  Path(id): Path<u64>,
) -> Result<Json<Ticket>> {
  let ticket = mc.delete_ticket(ctx, id).await?;
  Ok(Json(ticket))
}

pub fn routes(mc: ModelController) -> Router {
  Router::new()
    .route("/tickets", post(create_ticket).get(list_tickets))
    .route("/tickets/:id", delete(delete_ticket))
    .with_state(mc)
}
