mod api;
mod commands;
#[cfg(test)]
mod e2e;
mod error;
mod gateway;
mod jwt;
mod models;
mod secure_store;
mod session;
mod url;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "harmony_client_lib=debug".into()),
        )
        .try_init();

    tauri::Builder::default()
        .manage(session::SessionState::default())
        .invoke_handler(tauri::generate_handler![
            commands::session_restore,
            commands::session_login,
            commands::session_register,
            commands::session_logout,
            commands::list_users,
            commands::list_channels,
            commands::list_messages,
            commands::send_message,
            commands::edit_message,
            commands::delete_message,
            commands::set_invisible,
            commands::report_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
