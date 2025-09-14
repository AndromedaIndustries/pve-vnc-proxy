use crate::library;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = library::sql_lite::schema::session)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Session {
    pub id: i32,
    pub user_id: String,
    pub proxmox_node: String,
    pub proxmox_vm_id: String,
    pub proxmox_csrf_prevention_token: String,
    pub proxmox_auth_cookie: String,
    pub vnc_token: String,
    pub port: String,
}

#[derive(Insertable)]
#[diesel(table_name = library::sql_lite::schema::session)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSession<'a> {
    pub user_id: &'a String,
    pub service_id: &'a String,
    pub proxmox_node: &'a String,
    pub proxmox_vm_id: &'a String,
    pub proxmox_csrf_prevention_token: &'a String,
    pub proxmox_auth_cookie: &'a String,
    pub connection_date: &'a i64,
    pub vnc_password: String,
    pub vnc_token: String,
    pub port: &'a String,
}
