// @generated automatically by Diesel CLI.

diesel::table! {
    session (id) {
        id -> Integer,
        user_id -> Text,
        service_id -> Text,
        proxmox_node -> Text,
        proxmox_vm_id -> Text,
        proxmox_csrf_prevention_token -> Text,
        proxmox_auth_cookie -> Text,
        vnc_token -> Text,
        vnc_password -> Text,
        port -> Text,
        connection_date -> BigInt,
    }
}
