use std::net::IpAddr;

use actix_web::{HttpRequest, HttpResponse, get};
use serde_json::json;

fn build_info() -> serde_json::Value {
    json!({
        "name": "modrinth-labrinth",
        "version": env!("LABRINTH_VERSION"),
        "documentation": "https://docs.modrinth.com",
        "about": "Welcome traveler!",

        "build_info": {
            "comp_date": env!("COMPILATION_DATE"),
            "git_hash": env!("GIT_HASH"),
            "profile": env!("COMPILATION_PROFILE"),
        }
    })
}

#[get("/")]
pub async fn index_get() -> HttpResponse {
    HttpResponse::Ok().json(build_info())
}

#[get("/build")]
pub async fn build_get() -> HttpResponse {
    HttpResponse::Ok().json(build_info())
}

/// Liveness and readiness probe. Deliberately touches no backing services so a
/// Redis or Postgres failover cannot get healthy pods killed.
#[get("/healthz")]
pub async fn healthz_get(req: HttpRequest) -> HttpResponse {
    if !is_local_request(&req) {
        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Ok().finish()
}

/// Uses the TCP peer, never client-supplied headers. Requests carrying
/// forwarding headers came through a proxy, so their real origin is unknown.
fn is_local_request(req: &HttpRequest) -> bool {
    let headers = req.headers();
    if headers.contains_key("X-Forwarded-For")
        || headers.contains_key("CF-Connecting-IP")
    {
        return false;
    }

    let Some(peer) = req.peer_addr() else {
        return false;
    };

    match peer.ip().to_canonical() {
        IpAddr::V4(ip) => {
            ip.is_loopback() || ip.is_private() || ip.is_link_local()
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
        }
    }
}
