pub mod database;
pub mod app_state;
pub mod commands;
pub mod services;

use app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    rt.block_on(async {
        let app_state = AppState::new().await.expect("Failed to initialize app state");
        
        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .manage(app_state)
            .invoke_handler(tauri::generate_handler![
                commands::greet,
                commands::database_health_check,
                commands::create_repository,
                commands::get_repositories,
                commands::get_repository_by_id,
                commands::update_repository,
                commands::delete_repository,
                commands::create_workspace,
                commands::get_workspaces_by_repository,
                commands::archive_workspace,
                commands::restore_workspace,
                commands::delete_workspace,
                // Git operations
                commands::get_git_status,
                commands::get_git_branches,
                commands::create_git_worktree,
                commands::list_git_worktrees,
                commands::remove_git_worktree,
                commands::checkout_git_branch,
                commands::create_git_branch,
                commands::is_git_repository,
                commands::init_git_repository,
                commands::clone_git_repository,
                commands::set_global_gitignore,
                commands::get_global_gitignore,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
