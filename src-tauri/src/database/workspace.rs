use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::database::models::{Workspace, CreateWorkspaceRequest};

pub struct WorkspaceService {
    pool: SqlitePool,
}

impl WorkspaceService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateWorkspaceRequest, path: String) -> Result<Workspace> {
        let workspace = Workspace::new(
            req.repository_id,
            req.name,
            req.branch,
            path,
        );

        sqlx::query(
            r#"
            INSERT INTO workspaces (id, repository_id, name, branch, path, is_archived, created_at, updated_at, archived_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&workspace.id)
        .bind(&workspace.repository_id)
        .bind(&workspace.name)
        .bind(&workspace.branch)
        .bind(&workspace.path)
        .bind(&workspace.is_archived)
        .bind(&workspace.created_at)
        .bind(&workspace.updated_at)
        .bind(&workspace.archived_at)
        .execute(&self.pool)
        .await?;

        Ok(workspace)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Workspace>> {
        let workspace = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workspace)
    }

    pub async fn list_by_repository(&self, repository_id: &str) -> Result<Vec<Workspace>> {
        let workspaces = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE repository_id = $1 ORDER BY created_at DESC"
        )
        .bind(repository_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(workspaces)
    }

    pub async fn list_active_by_repository(&self, repository_id: &str) -> Result<Vec<Workspace>> {
        let workspaces = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE repository_id = $1 AND is_archived = FALSE ORDER BY created_at DESC"
        )
        .bind(repository_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(workspaces)
    }

    pub async fn list_archived_by_repository(&self, repository_id: &str) -> Result<Vec<Workspace>> {
        let workspaces = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE repository_id = $1 AND is_archived = TRUE ORDER BY archived_at DESC"
        )
        .bind(repository_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(workspaces)
    }

    pub async fn archive(&self, id: &str) -> Result<Option<Workspace>> {
        let mut workspace = match self.get_by_id(id).await? {
            Some(ws) => ws,
            None => return Ok(None),
        };

        workspace.archive();

        sqlx::query(
            "UPDATE workspaces SET is_archived = $1, archived_at = $2, updated_at = $3 WHERE id = $4"
        )
        .bind(&workspace.is_archived)
        .bind(&workspace.archived_at)
        .bind(&workspace.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(Some(workspace))
    }

    pub async fn restore(&self, id: &str) -> Result<Option<Workspace>> {
        let mut workspace = match self.get_by_id(id).await? {
            Some(ws) => ws,
            None => return Ok(None),
        };

        workspace.restore();

        sqlx::query(
            "UPDATE workspaces SET is_archived = $1, archived_at = $2, updated_at = $3 WHERE id = $4"
        )
        .bind(&workspace.is_archived)
        .bind(&workspace.archived_at)
        .bind(&workspace.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(Some(workspace))
    }

    pub async fn delete(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM workspaces WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists_by_name(&self, repository_id: &str, name: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM workspaces WHERE repository_id = $1 AND name = $2"
        )
        .bind(repository_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn update_branch(&self, id: &str, branch: &str) -> Result<Option<Workspace>> {
        let mut workspace = match self.get_by_id(id).await? {
            Some(ws) => ws,
            None => return Ok(None),
        };

        workspace.branch = branch.to_string();
        workspace.updated_at = Utc::now();

        sqlx::query("UPDATE workspaces SET branch = $1, updated_at = $2 WHERE id = $3")
            .bind(&workspace.branch)
            .bind(&workspace.updated_at)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(Some(workspace))
    }
}