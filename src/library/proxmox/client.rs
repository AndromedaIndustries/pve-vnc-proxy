use reqwest::header;
use serde::{Deserialize, Serialize};
use tracing;

#[derive(Debug, Deserialize)]
pub struct AuthTicketResponse {
    data: AuthTicketData,
}

#[derive(Debug, Deserialize)]
pub struct AuthTicketData {
    pub ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    pub csrf_prevention_token: String,
}

#[derive(Serialize)]
struct AuthTicketForm<'a> {
    username: &'a String,
    password: &'a String,
    realm: &'a String,
}
pub async fn get_pve_auth_ticket() -> Result<AuthTicketData, Box<dyn std::error::Error>> {
    let proxmox_host = std::env::var("PROXMOX_HOST")?;
    let proxmox_user = std::env::var("PROXMOX_USER")?;
    let proxmox_pass = std::env::var("PROXMOX_PASS")?;

    let client = reqwest::Client::new();
    // /api2/json/access/ticket
    let api_path =
        String::from("https://") + &proxmox_host + &String::from("/api2/json/access/ticket");

    tracing::debug!("API URL: {}", api_path);

    let form = AuthTicketForm {
        username: &proxmox_user,
        password: &proxmox_pass,
        realm: &String::from("pve"),
    };

    let resp = client
        .post(api_path)
        .form(&form)
        .send()
        .await?
        .json::<AuthTicketResponse>()
        .await?;

    Ok(resp.data)
}

#[derive(Debug, Deserialize)]
pub struct VncProxyResponse {
    data: VncProxyData,
}

#[derive(Debug, Deserialize)]
pub struct VncProxyData {
    pub port: String,
    pub ticket: String,
    pub password: String,
}

#[derive(Serialize)]
struct VncProxyParams {
    #[serde(rename = "generate-password", skip_serializing_if = "Option::is_none")]
    generate_password: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    websocket: Option<u8>,
}

pub async fn get_vnc_proxy(
    proxmox_node: String,
    proxmox_vm_id: String,
    proxmox_auth_token: String,
    csrf_prevention_token: String,
) -> Result<VncProxyData, Box<dyn std::error::Error>> {
    let proxmox_host = std::env::var("PROXMOX_HOST")?;

    let client = reqwest::Client::new();

    // /api2/json/nodes/{node}/qemu/{vmid}/vncproxy
    let api_path = String::from("https://")
        + &proxmox_host
        + &String::from("/api2/json/nodes/")
        + &proxmox_node
        + &String::from("/qemu/")
        + &proxmox_vm_id
        + &String::from("/vncproxy");

    tracing::debug!("API URL: {}", api_path);

    let params = VncProxyParams {
        generate_password: Some(1),
        websocket: Some(1),
    };

    tracing::debug!(
        "Parameters: Generate Password {} - websocket {}",
        params.generate_password.unwrap(),
        params.websocket.unwrap()
    );
    let pve_ticket = String::from("PVEAuthCookie=") + &proxmox_auth_token;

    let resp = client
        .post(api_path)
        .header(header::COOKIE, pve_ticket)
        .header("CSRFPreventionToken", csrf_prevention_token)
        .json(&params)
        .send()
        .await?
        .json::<VncProxyResponse>()
        .await?;

    tracing::debug!("Retrieved Proxmox VNC Port: {:?}", &resp.data.port);

    Ok(resp.data)
}
