use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::database::models::{Repository, CreateRepositoryRequest, UpdateRepositoryRequest};

pub struct RepositoryService {
    pool: SqlitePool,
}

impl RepositoryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateRepositoryRequest) -> Result<Repository> {
        let repository = Repository::new(
            req.name,
            req.path,
            req.source_branch,
            req.init_script,
        );

        sqlx::query(
            r#"
            INSERT INTO repositories (id, name, path, source_branch, init_script, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&repository.id)
        .bind(&repository.name)
        .bind(&repository.path)
        .bind(&repository.source_branch)
        .bind(&repository.init_script)
        .bind(&repository.created_at)
        .bind(&repository.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(repository)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Repository>> {
        let repository = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(repository)
    }

    pub async fn get_by_path(&self, path: &str) -> Result<Option<Repository>> {
        let repository = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE path = $1"
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?;

        Ok(repository)
    }

    pub async fn list_all(&self) -> Result<Vec<Repository>> {
        let repositories = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(repositories)
    }

    pub async fn update(&self, id: &str, req: UpdateRepositoryRequest) -> Result<Option<Repository>> {
        let mut repository = match self.get_by_id(id).await? {
            Some(repo) => repo,
            None => return Ok(None),
        };

        if let Some(name) = req.name {
            repository.name = name;
        }
        if let Some(source_branch) = req.source_branch {
            repository.source_branch = Some(source_branch);
        }
        if let Some(init_script) = req.init_script {
            repository.init_script = Some(init_script);
        }
        repository.updated_at = Utc::now();

        sqlx::query(
            r#"
            UPDATE repositories 
            SET name = $1, source_branch = $2, init_script = $3, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(&repository.name)
        .bind(&repository.source_branch)
        .bind(&repository.init_script)
        .bind(&repository.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(Some(repository))
    }

    pub async fn delete(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM repositories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists_by_path(&self, path: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM repositories WHERE path = $1")
            .bind(path)
            .fetch_one(&self.pool)
            .await?;

        Ok(count > 0)
    }
}