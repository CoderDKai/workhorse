use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Database, models::*};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let db = Database::new(&db_path).await?;
        assert!(db.health_check().await?);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_repository_crud() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let db = Database::new(&db_path).await?;
        let repo_service = crate::database::repository::RepositoryService::new(db.pool().clone());
        
        // Create repository
        let create_req = CreateRepositoryRequest {
            name: "test-repo".to_string(),
            path: "/path/to/repo".to_string(),
            source_branch: Some("main".to_string()),
            init_script: Some("echo 'hello'".to_string()),
        };
        
        let repo = repo_service.create(create_req).await?;
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.path, "/path/to/repo");
        
        // Get repository
        let fetched_repo = repo_service.get_by_id(&repo.id).await?;
        assert!(fetched_repo.is_some());
        assert_eq!(fetched_repo.unwrap().id, repo.id);
        
        // List repositories
        let repos = repo_service.list_all().await?;
        assert_eq!(repos.len(), 1);
        
        // Update repository
        let update_req = UpdateRepositoryRequest {
            name: Some("updated-repo".to_string()),
            source_branch: None,
            init_script: None,
        };
        
        let updated_repo = repo_service.update(&repo.id, update_req).await?;
        assert!(updated_repo.is_some());
        assert_eq!(updated_repo.unwrap().name, "updated-repo");
        
        // Delete repository
        let deleted = repo_service.delete(&repo.id).await?;
        assert!(deleted);
        
        let repos_after_delete = repo_service.list_all().await?;
        assert_eq!(repos_after_delete.len(), 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_workspace_crud() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let db = Database::new(&db_path).await?;
        let repo_service = crate::database::repository::RepositoryService::new(db.pool().clone());
        let workspace_service = crate::database::workspace::WorkspaceService::new(db.pool().clone());
        
        // Create repository first
        let create_repo_req = CreateRepositoryRequest {
            name: "test-repo".to_string(),
            path: "/path/to/repo".to_string(),
            source_branch: Some("main".to_string()),
            init_script: None,
        };
        
        let repo = repo_service.create(create_repo_req).await?;
        
        // Create workspace
        let create_ws_req = CreateWorkspaceRequest {
            repository_id: repo.id.clone(),
            name: "feature-branch".to_string(),
            branch: "feature/test".to_string(),
        };
        
        let workspace = workspace_service.create(create_ws_req, "/path/to/workspace".to_string()).await?;
        assert_eq!(workspace.name, "feature-branch");
        assert_eq!(workspace.branch, "feature/test");
        assert!(!workspace.is_archived);
        
        // Get workspace
        let fetched_ws = workspace_service.get_by_id(&workspace.id).await?;
        assert!(fetched_ws.is_some());
        
        // List workspaces by repository
        let workspaces = workspace_service.list_by_repository(&repo.id).await?;
        assert_eq!(workspaces.len(), 1);
        
        // Archive workspace
        let archived_ws = workspace_service.archive(&workspace.id).await?;
        assert!(archived_ws.is_some());
        assert!(archived_ws.unwrap().is_archived);
        
        // Restore workspace
        let restored_ws = workspace_service.restore(&workspace.id).await?;
        assert!(restored_ws.is_some());
        assert!(!restored_ws.unwrap().is_archived);
        
        // Delete workspace
        let deleted = workspace_service.delete(&workspace.id).await?;
        assert!(deleted);
        
        let workspaces_after_delete = workspace_service.list_by_repository(&repo.id).await?;
        assert_eq!(workspaces_after_delete.len(), 0);
        
        Ok(())
    }
}