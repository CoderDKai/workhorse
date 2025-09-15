use tauri::State;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::database::models::{Repository, Workspace, CreateRepositoryRequest, CreateWorkspaceRequest, UpdateRepositoryRequest};
use crate::services::{GitService, RepositoryManagerService};
use crate::services::git_service::{GitStatus, GitBranch, WorktreeInfo};
use crate::services::repository_service::{RepositoryConfig, RepositoryValidationResult, AddRepositoryRequest, RepositoryScript};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub async fn database_health_check(state: State<'_, AppState>) -> Result<ApiResponse<bool>, String> {
    match state.database.health_check().await {
        Ok(healthy) => Ok(ApiResponse::success(healthy)),
        Err(e) => Ok(ApiResponse::error(format!("Database health check failed: {}", e))),
    }
}

#[tauri::command]
pub async fn create_repository(
    state: State<'_, AppState>,
    request: CreateRepositoryRequest,
) -> Result<ApiResponse<Repository>, String> {
    match state.repository_service.create(request).await {
        Ok(repository) => Ok(ApiResponse::success(repository)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to create repository: {}", e))),
    }
}

#[tauri::command]
pub async fn get_repositories(state: State<'_, AppState>) -> Result<ApiResponse<Vec<Repository>>, String> {
    match state.repository_service.list_all().await {
        Ok(repositories) => Ok(ApiResponse::success(repositories)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get repositories: {}", e))),
    }
}

#[tauri::command]
pub async fn get_repository_by_id(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<Option<Repository>>, String> {
    match state.repository_service.get_by_id(&id).await {
        Ok(repository) => Ok(ApiResponse::success(repository)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get repository: {}", e))),
    }
}

#[tauri::command]
pub async fn update_repository(
    state: State<'_, AppState>,
    id: String,
    request: UpdateRepositoryRequest,
) -> Result<ApiResponse<Option<Repository>>, String> {
    match state.repository_service.update(&id, request).await {
        Ok(repository) => Ok(ApiResponse::success(repository)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to update repository: {}", e))),
    }
}

#[tauri::command]
pub async fn delete_repository(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<bool>, String> {
    match state.repository_service.delete(&id).await {
        Ok(deleted) => Ok(ApiResponse::success(deleted)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to delete repository: {}", e))),
    }
}

#[tauri::command]
pub async fn create_workspace(
    state: State<'_, AppState>,
    request: CreateWorkspaceRequest,
    path: String,
) -> Result<ApiResponse<Workspace>, String> {
    match state.workspace_service.create(request, path).await {
        Ok(workspace) => Ok(ApiResponse::success(workspace)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to create workspace: {}", e))),
    }
}

#[tauri::command]
pub async fn get_workspaces_by_repository(
    state: State<'_, AppState>,
    repository_id: String,
) -> Result<ApiResponse<Vec<Workspace>>, String> {
    match state.workspace_service.list_by_repository(&repository_id).await {
        Ok(workspaces) => Ok(ApiResponse::success(workspaces)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get workspaces: {}", e))),
    }
}

#[tauri::command]
pub async fn archive_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<Option<Workspace>>, String> {
    match state.workspace_service.archive(&id).await {
        Ok(workspace) => Ok(ApiResponse::success(workspace)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to archive workspace: {}", e))),
    }
}

#[tauri::command]
pub async fn restore_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<Option<Workspace>>, String> {
    match state.workspace_service.restore(&id).await {
        Ok(workspace) => Ok(ApiResponse::success(workspace)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to restore workspace: {}", e))),
    }
}

#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<bool>, String> {
    match state.workspace_service.delete(&id).await {
        Ok(deleted) => Ok(ApiResponse::success(deleted)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to delete workspace: {}", e))),
    }
}

// Git Operations Commands

#[tauri::command]
pub async fn get_git_status(repo_path: String) -> Result<ApiResponse<GitStatus>, String> {
    match GitService::get_repository_status(&repo_path) {
        Ok(status) => Ok(ApiResponse::success(status)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get git status: {}", e))),
    }
}

#[tauri::command]
pub async fn get_git_branches(repo_path: String) -> Result<ApiResponse<Vec<GitBranch>>, String> {
    match GitService::get_branches(&repo_path) {
        Ok(branches) => Ok(ApiResponse::success(branches)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get branches: {}", e))),
    }
}

#[tauri::command]
pub async fn create_git_worktree(
    repo_path: String,
    worktree_name: String,
    worktree_path: String,
    branch_name: Option<String>,
) -> Result<ApiResponse<bool>, String> {
    match GitService::create_worktree(&repo_path, &worktree_name, &worktree_path, branch_name.as_deref()) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to create worktree: {}", e))),
    }
}

#[tauri::command]
pub async fn list_git_worktrees(repo_path: String) -> Result<ApiResponse<Vec<WorktreeInfo>>, String> {
    match GitService::list_worktrees(&repo_path) {
        Ok(worktrees) => Ok(ApiResponse::success(worktrees)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to list worktrees: {}", e))),
    }
}

#[tauri::command]
pub async fn remove_git_worktree(
    repo_path: String,
    worktree_name: String,
) -> Result<ApiResponse<bool>, String> {
    match GitService::remove_worktree(&repo_path, &worktree_name) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to remove worktree: {}", e))),
    }
}

#[tauri::command]
pub async fn checkout_git_branch(
    repo_path: String,
    branch_name: String,
) -> Result<ApiResponse<bool>, String> {
    match GitService::checkout_branch(&repo_path, &branch_name) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to checkout branch: {}", e))),
    }
}

#[tauri::command]
pub async fn create_git_branch(
    repo_path: String,
    branch_name: String,
    from_branch: Option<String>,
) -> Result<ApiResponse<bool>, String> {
    match GitService::create_branch(&repo_path, &branch_name, from_branch.as_deref()) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to create branch: {}", e))),
    }
}

#[tauri::command]
pub async fn is_git_repository(path: String) -> Result<ApiResponse<bool>, String> {
    let is_repo = GitService::is_git_repository(&path);
    Ok(ApiResponse::success(is_repo))
}

#[tauri::command]
pub async fn init_git_repository(path: String, bare: bool) -> Result<ApiResponse<bool>, String> {
    match GitService::init_repository(&path, bare) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to initialize repository: {}", e))),
    }
}

#[tauri::command]
pub async fn clone_git_repository(
    url: String,
    path: String,
) -> Result<ApiResponse<bool>, String> {
    match GitService::clone_repository(&url, &path, None) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to clone repository: {}", e))),
    }
}

#[tauri::command]
pub async fn set_global_gitignore(gitignore_path: String) -> Result<ApiResponse<bool>, String> {
    match GitService::set_global_gitignore(&gitignore_path) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to set global gitignore: {}", e))),
    }
}

#[tauri::command]
pub async fn get_global_gitignore() -> Result<ApiResponse<Option<String>>, String> {
    match GitService::get_global_gitignore() {
        Ok(path) => Ok(ApiResponse::success(path)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get global gitignore: {}", e))),
    }
}

// Repository Management Commands

#[tauri::command]
pub async fn validate_repository(repo_path: String) -> Result<ApiResponse<RepositoryValidationResult>, String> {
    match RepositoryManagerService::validate_repository(&repo_path) {
        Ok(result) => Ok(ApiResponse::success(result)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to validate repository: {}", e))),
    }
}

#[tauri::command]
pub async fn add_repository_management(
    request: AddRepositoryRequest,
) -> Result<ApiResponse<RepositoryConfig>, String> {
    match RepositoryManagerService::add_repository(request) {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to add repository: {}", e))),
    }
}

#[tauri::command]
pub async fn load_repository_config(repo_path: String) -> Result<ApiResponse<RepositoryConfig>, String> {
    match RepositoryManagerService::load_repository_config(&repo_path) {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to load repository config: {}", e))),
    }
}

#[tauri::command]
pub async fn is_managed_repository(repo_path: String) -> Result<ApiResponse<bool>, String> {
    let is_managed = RepositoryManagerService::is_managed_repository(&repo_path);
    Ok(ApiResponse::success(is_managed))
}

#[tauri::command]
pub async fn remove_repository_management(repo_path: String) -> Result<ApiResponse<bool>, String> {
    match RepositoryManagerService::remove_repository_management(&repo_path) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to remove repository management: {}", e))),
    }
}

#[tauri::command]
pub async fn add_repository_script(
    repo_path: String,
    script: RepositoryScript,
) -> Result<ApiResponse<RepositoryConfig>, String> {
    match RepositoryManagerService::add_script(&repo_path, script) {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to add script: {}", e))),
    }
}

#[tauri::command]
pub async fn remove_repository_script(
    repo_path: String,
    script_name: String,
) -> Result<ApiResponse<RepositoryConfig>, String> {
    match RepositoryManagerService::remove_script(&repo_path, &script_name) {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to remove script: {}", e))),
    }
}

#[tauri::command]
pub async fn get_repository_scripts(repo_path: String) -> Result<ApiResponse<Vec<RepositoryScript>>, String> {
    match RepositoryManagerService::get_scripts(&repo_path) {
        Ok(scripts) => Ok(ApiResponse::success(scripts)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to get scripts: {}", e))),
    }
}

#[tauri::command]
pub async fn create_workhorse_directory(repo_path: String) -> Result<ApiResponse<String>, String> {
    match RepositoryManagerService::create_workhorse_directory(&repo_path) {
        Ok(workhorse_dir) => Ok(ApiResponse::success(workhorse_dir.to_string_lossy().to_string())),
        Err(e) => Ok(ApiResponse::error(format!("Failed to create workhorse directory: {}", e))),
    }
}

#[tauri::command]
pub async fn cleanup_repository_temp_files(repo_path: String) -> Result<ApiResponse<bool>, String> {
    match RepositoryManagerService::cleanup_temp_files(&repo_path) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to cleanup temp files: {}", e))),
    }
}

#[tauri::command]
pub async fn get_repository_directories(repo_path: String) -> Result<ApiResponse<serde_json::Value>, String> {
    let directories = serde_json::json!({
        "workhorse": RepositoryManagerService::get_workspaces_dir(&repo_path).to_string_lossy(),
        "workspaces": RepositoryManagerService::get_workspaces_dir(&repo_path).to_string_lossy(),
        "configs": RepositoryManagerService::get_configs_dir(&repo_path).to_string_lossy(),
        "scripts": RepositoryManagerService::get_scripts_dir(&repo_path).to_string_lossy(),
        "logs": RepositoryManagerService::get_logs_dir(&repo_path).to_string_lossy(),
        "temp": RepositoryManagerService::get_temp_dir(&repo_path).to_string_lossy(),
    });
    
    Ok(ApiResponse::success(directories))
}