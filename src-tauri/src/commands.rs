use tauri::State;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::database::models::{Repository, Workspace, CreateRepositoryRequest, CreateWorkspaceRequest, UpdateRepositoryRequest};

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