-- Your SQL goes here
CREATE TABLE session (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    proxmox_node TEXT NOT NULL,
    proxmox_vm_id TEXT NOT NULL,
    proxmox_csrf_prevention_token TEXT NOT NULL,
    proxmox_auth_cookie TEXT NOT NULL,
    vnc_token TEXT NOT NULL,
    vnc_password TEXT NOT NULL,
    port TEXT NOT NULL,
    connection_date BIGINT NOT NULL
);