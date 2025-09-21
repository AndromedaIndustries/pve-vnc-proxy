use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::header::{AUTHORIZATION, HeaderMap},
    post, web,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{self, warn};

use crate::library::{
    self,
    proxmox::client::{get_pve_auth_ticket, get_vnc_proxy},
    sql_lite::client::insert_session,
};

#[derive(Deserialize)]
struct GetToken {
    service_id: String,
}

#[derive(Serialize)]
struct SessionIdReturn {
    session_id: i32,
    status: String,
    password: String,
}

#[post("/api/request/session/id")]
pub async fn get_session_id(req: HttpRequest, data: web::Json<GetToken>) -> impl Responder {
    let headers: &HeaderMap = req.headers();
    let user_id;

    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            let validator_response = library::jwt::validation::validator(auth_str).await;

            let claims = match validator_response {
                Ok(c) => c,
                Err(_) => {
                    warn!("Failed to validate claim");
                    return HttpResponse::Unauthorized().body("Unauthorized");
                }
            };

            user_id = claims.user_id().to_string();
        } else {
            warn!("Failed to get auth string");
            return HttpResponse::Unauthorized().body("Unauthorized");
        }
    } else {
        warn!("Auth Header Missing");
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    if user_id.is_empty() {
        warn!("User not authorized to access service");
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    tracing::debug!("Successful JWT validation");

    let supabase = library::db::client::new().await;

    if supabase.is_err() {
        HttpResponse::InternalServerError();
    }

    let supabase_client = supabase.unwrap();

    let service: library::supabase::public::Service =
        sqlx::query_as::<_, library::supabase::public::Service>(
            r#"SELECT * FROM "Services" WHERE user_id = $1::uuid AND id = $2::text"#,
        )
        .bind(user_id.clone())
        .bind(&data.service_id)
        .fetch_one(&supabase_client)
        .await
        .unwrap();

    tracing::debug!("Gathered service");

    let proxmox_vm_id = service.proxmox_vm_id.unwrap();
    let proxmox_node = service.proxmox_node.unwrap();
    let now = Utc::now();

    let auth_data = get_pve_auth_ticket().await.unwrap();

    let vnc_proxy_data = get_vnc_proxy(
        proxmox_node.clone(),
        proxmox_vm_id.clone(),
        auth_data.ticket.clone(),
        auth_data.csrf_prevention_token.clone(),
    )
    .await
    .unwrap();

    let new_session = library::sql_lite::models::NewSession {
        proxmox_vm_id: &proxmox_vm_id,
        proxmox_node: &proxmox_node,
        proxmox_auth_cookie: &auth_data.ticket,
        proxmox_csrf_prevention_token: &auth_data.csrf_prevention_token,
        service_id: &data.service_id,
        user_id: &user_id,
        connection_date: &now.timestamp_millis(),
        vnc_password: String::from(""),
        vnc_token: vnc_proxy_data.ticket,
        port: &vnc_proxy_data.port,
    };

    let mut sql_lite = library::sql_lite::client::new();

    let session = insert_session(&mut sql_lite, new_session);

    let data = web::Json(SessionIdReturn {
        session_id: session.id,
        status: String::from("200"),
        password: vnc_proxy_data.password,
    });

    HttpResponse::Ok().json(data)
}
