use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub source_branch: Option<String>,
    pub init_script: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workspace {
    pub id: String,
    pub repository_id: String,
    pub name: String,
    pub branch: String,
    pub path: String,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub path: String,
    pub source_branch: Option<String>,
    pub init_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub repository_id: String,
    pub name: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRepositoryRequest {
    pub name: Option<String>,
    pub source_branch: Option<String>,
    pub init_script: Option<String>,
}

impl Repository {
    pub fn new(name: String, path: String, source_branch: Option<String>, init_script: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            path,
            source_branch,
            init_script,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Workspace {
    pub fn new(repository_id: String, name: String, branch: String, path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            repository_id,
            name,
            branch,
            path,
            is_archived: false,
            created_at: now,
            updated_at: now,
            archived_at: None,
        }
    }

    pub fn archive(&mut self) {
        self.is_archived = true;
        self.archived_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn restore(&mut self) {
        self.is_archived = false;
        self.archived_at = None;
        self.updated_at = Utc::now();
    }
}