use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool, Row};
use std::path::Path;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_path: &Path) -> Result<Self> {
        // 确保数据库目录存在
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;
        
        let db = Self { pool };
        db.migrate().await?;
        
        Ok(db)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn migrate(&self) -> Result<()> {
        // 创建仓库表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS repositories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                source_branch TEXT,
                init_script TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建工作区表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                name TEXT NOT NULL,
                branch TEXT NOT NULL,
                path TEXT NOT NULL,
                is_archived BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                archived_at TEXT,
                FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
                UNIQUE(repository_id, name)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspaces_repository_id ON workspaces(repository_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspaces_archived ON workspaces(is_archived)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        let result = sqlx::query("SELECT 1 as health")
            .fetch_one(&self.pool)
            .await?;
        
        let health: i32 = result.get("health");
        Ok(health == 1)
    }
}