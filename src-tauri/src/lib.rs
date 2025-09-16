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
                // Repository management
                commands::validate_repository,
                commands::add_repository_management,
                commands::load_repository_config,
                commands::is_managed_repository,
                commands::remove_repository_management,
                commands::add_repository_script,
                commands::remove_repository_script,
                commands::get_repository_scripts,
                commands::create_workhorse_directory,
                commands::cleanup_repository_temp_files,
                commands::get_repository_directories,
                // Workspace management
                commands::create_managed_workspace,
                commands::list_managed_workspaces,
                commands::get_managed_workspace_info,
                commands::archive_managed_workspace,
                commands::restore_managed_workspace,
                commands::delete_managed_workspace,
                commands::update_managed_workspace_status,
                commands::access_managed_workspace,
                commands::add_workspace_tag,
                commands::remove_workspace_tag,
                commands::set_workspace_custom_field,
                commands::remove_workspace_custom_field,
                commands::find_workspaces_by_tag,
                commands::find_workspaces_by_status,
                commands::cleanup_broken_workspaces,
                commands::get_workspace_statistics,
                // Script execution
                commands::create_script_execution,
                commands::execute_script,
                commands::cancel_script_execution,
                commands::get_script_execution_status,
                commands::get_all_script_executions,
                commands::cleanup_completed_script_executions,
                // Terminal service
                commands::create_terminal,
                commands::start_terminal,
                commands::send_terminal_command,
                commands::execute_single_command,
                commands::get_terminal_output,
                commands::get_terminal_history,
                commands::close_terminal,
                commands::get_terminal_session,
                commands::get_all_terminals,
                commands::cleanup_closed_terminals,
                commands::set_terminal_name,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
