use chrono::NaiveDateTime;
use serde_json::Value;
use uuid::Uuid;

/// Services
#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct Service {
    pub id: String,
    pub user_id: Uuid,
    pub service_type: String,
    pub hostname: String,
    pub template_id: String,
    pub os_name: String,
    pub os_version: String,
    pub public_key_id: String,
    pub username: String,
    pub sku_id: String,
    pub current_sku_id: String,
    pub initial_sku_id: String,
    pub subscription_id: Option<String>,
    pub initial_checkout_id: Option<String>,
    pub status: String,
    pub status_reason: Option<String>,
    pub payment_ids: Option<Vec<String>>,
    pub payment_status: Option<String>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<String>,
    pub proxmox_node: Option<String>,
    pub proxmox_vm_id: Option<String>,
    pub metadata: Value,
    pub service_active: bool,
    pub subscription_active: bool,
    pub created_at: NaiveDateTime,
}
