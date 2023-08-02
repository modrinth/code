use crate::database::models::flow_item::Flow;
use crate::queue::socket::ActiveSockets;
use actix_web::web::Payload;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{Closed, Session};
use chrono::Duration;
use tokio::sync::RwLock;

#[get("ws")]
pub async fn route(
    req: HttpRequest,
    body: Payload,
    db: web::Data<RwLock<ActiveSockets>>,
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, session, _msg_stream) = actix_ws::handle(&req, body)?;
    let _ = sock(session, db, redis).await;

    Ok(res)
}

async fn sock(
    mut ws_stream: Session,
    db: web::Data<RwLock<ActiveSockets>>,
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<(), Closed> {
    if let Ok(state) = Flow::MinecraftAuth
        .insert(Duration::minutes(30), &redis)
        .await
    {
        ws_stream
            .text(serde_json::json!({ "login_code": state }).to_string())
            .await?;

        let db = db.write().await;
        db.auth_sockets.insert(state, ws_stream);
    }

    Ok(())
}
