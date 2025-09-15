use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::database::{Database, repository::RepositoryService, workspace::WorkspaceService};

pub struct AppState {
    pub database: Arc<Database>,
    pub repository_service: Arc<RepositoryService>,
    pub workspace_service: Arc<WorkspaceService>,
    pub data_dir: Arc<RwLock<PathBuf>>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // 获取应用数据目录
        let data_dir = get_app_data_dir()?;
        let db_path = data_dir.join("workhorse.db");
        
        // 初始化数据库
        let database = Arc::new(Database::new(&db_path).await?);
        
        // 创建服务
        let repository_service = Arc::new(RepositoryService::new(database.pool().clone()));
        let workspace_service = Arc::new(WorkspaceService::new(database.pool().clone()));
        
        Ok(Self {
            database,
            repository_service,
            workspace_service,
            data_dir: Arc::new(RwLock::new(data_dir)),
        })
    }
}

fn get_app_data_dir() -> Result<PathBuf> {
    // 在用户主目录下创建 .workhorse 目录
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    
    let data_dir = home_dir.join(".workhorse");
    std::fs::create_dir_all(&data_dir)?;
    
    Ok(data_dir)
}

// 为了获取家目录，我们需要添加 dirs 依赖