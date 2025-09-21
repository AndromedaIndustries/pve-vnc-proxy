use actix_web::{
    HttpRequest, HttpResponse, Result, get,
    web::{self},
};
use actix_ws::Message;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures_util::{SinkExt, StreamExt};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rand::RngCore;
use reqwest::header::{self, HeaderValue};
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::{Message as TMsg, http::Request as WsRequest};
use tracing::{debug, info};
use url::Url;

use crate::library::{self, sql_lite::client::delete_session};

type Payload = web::Payload;

#[get("/ws")]
async fn ws_proxy(
    req: HttpRequest,
    payload: Payload,
    q: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let (response, session, msg_stream) = actix_ws::handle(&req, payload)?;
    let get_session_id = q.get("session_id").cloned();
    let get_token = q.get("token").cloned();

    if get_token.is_none() {
        return Ok(HttpResponse::BadRequest().body("Missing required Parameters"));
    };

    let auth = String::from("Bearer ") + &get_token.unwrap();

    let validator_response = library::jwt::validation::validator(&auth).await;

    let claims = match validator_response {
        Ok(c) => c,
        Err(_) => return Ok(HttpResponse::Unauthorized().body("Unauthorized")),
    };

    let user_id = claims.user_id().to_string();

    if get_session_id.is_none() {
        return Ok(HttpResponse::BadRequest().body("Missing required Parameters"));
    };

    let session_id = get_session_id.unwrap();

    let proxmox_host = std::env::var("PROXMOX_HOST").unwrap();
    let conn = &mut library::sql_lite::client::new();

    let session_data = library::sql_lite::client::get_session(conn, session_id.clone());

    if session_data.user_id != user_id {
        return Ok(HttpResponse::BadRequest().body("Invalid Claim"));
    }

    let vnc_ticket_q = utf8_percent_encode(&session_data.vnc_token, NON_ALPHANUMERIC).to_string();

    let prox_url = format!(
        "wss://{host}/api2/json/nodes/{node}/qemu/{vm_id}/vncwebsocket?port={port}&vncticket={vncticket}",
        host = &proxmox_host,
        node = session_data.proxmox_node,
        vm_id = session_data.proxmox_vm_id,
        port = session_data.port,
        vncticket = vnc_ticket_q,
    );

    debug!("Proxmox WSS URL: {}", prox_url);

    let url = Url::parse(&prox_url).unwrap();

    let pve_ticket = String::from("PVEAuthCookie=") + &session_data.proxmox_auth_cookie;
    let csrf_prevention_token = session_data.proxmox_csrf_prevention_token;

    let mut nonce = [0u8; 16];
    rand::rng().fill_bytes(&mut nonce);
    let key = STANDARD.encode(nonce);

    let ws_request = WsRequest::builder()
        .uri(url.as_str())
        .header(header::COOKIE, pve_ticket)
        .header("CSRFPreventionToken", csrf_prevention_token)
        .header("X-Requested-By", "actix-proxy")
        .header("Host", format!("{proxmox_host}:8006"))
        .header("Origin", format!("https://{proxmox_host}:8006"))
        .header("Sec-WebSocket-Protocol", "binary")
        .header(header::SEC_WEBSOCKET_KEY, &key)
        .header(
            header::SEC_WEBSOCKET_VERSION,
            HeaderValue::from_static("13"),
        )
        .header(header::CONNECTION, "Upgrade")
        .header(header::UPGRADE, "websocket")
        .body(())
        .unwrap();

    info!("Starting Proxmox Websocket Connection");

    let (remote_ws, _) = tokio_tungstenite::connect_async(ws_request)
        .await
        .map_err(|e| {
            eprintln!("Failed to dial Proxmox WS: {:?}", e);
            actix_web::error::ErrorBadGateway("Proxmox WS error")
        })?;

    info!("Creating Proxmox Websocket Streams");

    let (mut remote_write, mut remote_read) = remote_ws.split();

    info!("Started Proxmox Websocket Connection");

    // client → Proxmox
    actix_web::rt::spawn({
        let mut incoming = msg_stream;
        async move {
            while let Some(Ok(msg)) = incoming.next().await {
                match msg {
                    Message::Text(t) => {
                        let s = t.to_string();
                        if remote_write.send(TMsg::Text(s.into())).await.is_err() {
                            break;
                        }
                    }
                    Message::Binary(b) => {
                        if remote_write.send(TMsg::Binary(b)).await.is_err() {
                            break;
                        };
                    }
                    Message::Ping(b) => {
                        let _ = remote_write.send(TMsg::Ping(b)).await;
                    }
                    Message::Close(_) => {
                        let _ = remote_write.send(TMsg::Close(None)).await;
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    // Proxmox → client
    actix_web::rt::spawn({
        let mut session = session;
        async move {
            while let Some(result) = remote_read.next().await {
                match result {
                    Ok(TMsg::Text(txt)) => {
                        let _ = session.text(txt.as_ref()).await;
                    }
                    Ok(TMsg::Binary(bin)) => {
                        let _ = session.binary(bin).await;
                    }
                    Ok(TMsg::Ping(buf)) => {
                        let _ = session.ping(&buf).await;
                    }
                    Ok(TMsg::Close(_)) => {
                        let _ = session.close(None).await;
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    });

    // This cleans up the session as you'll need to re-request a token to get a new session.
    delete_session(conn, session_id);

    Ok(response)
}
