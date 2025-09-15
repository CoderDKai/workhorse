use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use crate::services::{GitService, RepositoryManagerService};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkspaceStatus {
    Active,      // 工作区正在使用
    Inactive,    // 工作区存在但未使用
    Archived,    // 工作区已归档
    Broken,      // 工作区损坏或路径不存在
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub id: String,
    pub name: String,
    pub repository_path: PathBuf,
    pub workspace_path: PathBuf,
    pub branch: Option<String>,
    pub status: WorkspaceStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub repository_path: PathBuf,
    pub branch: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub base_path: Option<PathBuf>,  // 工作区的基础路径，如果不指定则使用默认位置
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub metadata: WorkspaceMetadata,
    pub git_status: Option<crate::services::git_service::GitStatus>,
    pub path_exists: bool,
    pub is_git_worktree: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveWorkspaceRequest {
    pub workspace_id: String,
    pub keep_files: bool,  // 是否保留工作区文件
    pub archive_reason: Option<String>,
}

pub struct WorkspaceManagerService;

impl WorkspaceManagerService {
    pub fn new() -> Self {
        Self
    }

    /// 创建新的工作区
    pub fn create_workspace(
        repo_path: &Path,
        request: CreateWorkspaceRequest,
    ) -> Result<WorkspaceMetadata> {
        // 验证仓库是否被管理
        if !RepositoryManagerService::is_managed_repository(repo_path) {
            return Err(anyhow!("仓库未被Workhorse管理，请先添加仓库管理"));
        }

        // 验证仓库是否为有效的Git仓库
        if !GitService::is_git_repository(repo_path) {
            return Err(anyhow!("指定路径不是有效的Git仓库"));
        }

        // 生成工作区ID
        let workspace_id = uuid::Uuid::new_v4().to_string();

        // 确定工作区路径
        let workspace_path = Self::determine_workspace_path(repo_path, &request)?;

        // 确保工作区路径不存在
        if workspace_path.exists() {
            return Err(anyhow!("工作区路径已存在: {:?}", workspace_path));
        }

        // 创建Git worktree
        let worktree_name = format!("ws-{}", &workspace_id[..8]);
        GitService::create_worktree(
            repo_path,
            &worktree_name,
            &workspace_path,
            request.branch.as_deref(),
        )?;

        let now = chrono::Utc::now();
        let metadata = WorkspaceMetadata {
            id: workspace_id,
            name: request.name,
            repository_path: repo_path.to_path_buf(),
            workspace_path,
            branch: request.branch,
            status: WorkspaceStatus::Active,
            created_at: now,
            updated_at: now,
            last_accessed_at: Some(now),
            archived_at: None,
            description: request.description,
            tags: request.tags,
            custom_fields: HashMap::new(),
        };

        // 保存工作区元数据
        Self::save_workspace_metadata(repo_path, &metadata)?;

        // 更新工作区索引
        Self::update_workspace_index(repo_path)?;

        Ok(metadata)
    }

    /// 确定工作区路径
    fn determine_workspace_path(
        repo_path: &Path,
        request: &CreateWorkspaceRequest,
    ) -> Result<PathBuf> {
        let base_path = if let Some(base) = &request.base_path {
            base.clone()
        } else {
            // 默认在仓库同级目录创建工作区
            repo_path
                .parent()
                .ok_or_else(|| anyhow!("无法确定仓库父目录"))?
                .to_path_buf()
        };

        let repo_name = repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("无法获取仓库名称"))?;

        let workspace_dir_name = format!("{}-{}", repo_name, request.name);
        Ok(base_path.join(workspace_dir_name))
    }

    /// 保存工作区元数据
    fn save_workspace_metadata(repo_path: &Path, metadata: &WorkspaceMetadata) -> Result<()> {
        let workspaces_dir = RepositoryManagerService::get_workspaces_dir(repo_path);
        fs::create_dir_all(&workspaces_dir)?;

        let metadata_file = workspaces_dir.join(format!("{}.json", metadata.id));
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_file, metadata_json)?;

        Ok(())
    }

    /// 加载工作区元数据
    pub fn load_workspace_metadata(repo_path: &Path, workspace_id: &str) -> Result<WorkspaceMetadata> {
        let workspaces_dir = RepositoryManagerService::get_workspaces_dir(repo_path);
        let metadata_file = workspaces_dir.join(format!("{}.json", workspace_id));

        if !metadata_file.exists() {
            return Err(anyhow!("工作区元数据文件不存在"));
        }

        let metadata_content = fs::read_to_string(&metadata_file)?;
        let metadata: WorkspaceMetadata = serde_json::from_str(&metadata_content)?;

        Ok(metadata)
    }

    /// 获取所有工作区列表
    pub fn list_workspaces(repo_path: &Path) -> Result<Vec<WorkspaceMetadata>> {
        let workspaces_dir = RepositoryManagerService::get_workspaces_dir(repo_path);
        
        if !workspaces_dir.exists() {
            return Ok(Vec::new());
        }

        let mut workspaces = Vec::new();
        
        for entry in fs::read_dir(&workspaces_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata_content) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<WorkspaceMetadata>(&metadata_content) {
                        workspaces.push(metadata);
                    }
                }
            }
        }

        // 按创建时间排序
        workspaces.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(workspaces)
    }

    /// 获取工作区详细信息
    pub fn get_workspace_info(repo_path: &Path, workspace_id: &str) -> Result<WorkspaceInfo> {
        let metadata = Self::load_workspace_metadata(repo_path, workspace_id)?;
        
        let path_exists = metadata.workspace_path.exists();
        let is_git_worktree = if path_exists {
            GitService::is_git_repository(&metadata.workspace_path)
        } else {
            false
        };

        let git_status = if is_git_worktree {
            GitService::get_repository_status(&metadata.workspace_path).ok()
        } else {
            None
        };

        Ok(WorkspaceInfo {
            metadata,
            git_status,
            path_exists,
            is_git_worktree,
        })
    }

    /// 归档工作区
    pub fn archive_workspace(
        repo_path: &Path,
        request: ArchiveWorkspaceRequest,
    ) -> Result<WorkspaceMetadata> {
        let mut metadata = Self::load_workspace_metadata(repo_path, &request.workspace_id)?;

        if metadata.status == WorkspaceStatus::Archived {
            return Err(anyhow!("工作区已经被归档"));
        }

        // 如果不保留文件，删除工作区目录
        if !request.keep_files && metadata.workspace_path.exists() {
            // 先移除Git worktree
            let worktree_name = format!("ws-{}", &metadata.id[..8]);
            if let Err(e) = GitService::remove_worktree(repo_path, &worktree_name) {
                eprintln!("警告: 移除Git worktree失败: {}", e);
            }

            // 删除工作区目录
            fs::remove_dir_all(&metadata.workspace_path)?;
        }

        // 更新元数据
        metadata.status = WorkspaceStatus::Archived;
        metadata.archived_at = Some(chrono::Utc::now());
        metadata.updated_at = chrono::Utc::now();

        // 添加归档原因到自定义字段
        if let Some(reason) = request.archive_reason {
            metadata.custom_fields.insert("archive_reason".to_string(), reason);
        }

        // 保存更新后的元数据
        Self::save_workspace_metadata(repo_path, &metadata)?;

        // 更新工作区索引
        Self::update_workspace_index(repo_path)?;

        Ok(metadata)
    }

    /// 恢复工作区
    pub fn restore_workspace(repo_path: &Path, workspace_id: &str) -> Result<WorkspaceMetadata> {
        let mut metadata = Self::load_workspace_metadata(repo_path, workspace_id)?;

        if metadata.status != WorkspaceStatus::Archived {
            return Err(anyhow!("只能恢复已归档的工作区"));
        }

        // 检查工作区路径是否存在
        let path_exists = metadata.workspace_path.exists();

        if !path_exists {
            // 重新创建Git worktree
            let worktree_name = format!("ws-{}", &metadata.id[..8]);
            GitService::create_worktree(
                repo_path,
                &worktree_name,
                &metadata.workspace_path,
                metadata.branch.as_deref(),
            )?;
        }

        // 更新元数据
        metadata.status = WorkspaceStatus::Active;
        metadata.archived_at = None;
        metadata.updated_at = chrono::Utc::now();
        metadata.last_accessed_at = Some(chrono::Utc::now());

        // 移除归档原因
        metadata.custom_fields.remove("archive_reason");

        // 保存更新后的元数据
        Self::save_workspace_metadata(repo_path, &metadata)?;

        // 更新工作区索引
        Self::update_workspace_index(repo_path)?;

        Ok(metadata)
    }

    /// 删除工作区
    pub fn delete_workspace(repo_path: &Path, workspace_id: &str) -> Result<()> {
        let metadata = Self::load_workspace_metadata(repo_path, workspace_id)?;

        // 删除Git worktree（如果存在）
        let worktree_name = format!("ws-{}", &workspace_id[..8]);
        if let Err(e) = GitService::remove_worktree(repo_path, &worktree_name) {
            eprintln!("警告: 移除Git worktree失败: {}", e);
        }

        // 删除工作区目录（如果存在）
        if metadata.workspace_path.exists() {
            fs::remove_dir_all(&metadata.workspace_path)?;
        }

        // 删除元数据文件
        let workspaces_dir = RepositoryManagerService::get_workspaces_dir(repo_path);
        let metadata_file = workspaces_dir.join(format!("{}.json", workspace_id));
        if metadata_file.exists() {
            fs::remove_file(metadata_file)?;
        }

        // 更新工作区索引
        Self::update_workspace_index(repo_path)?;

        Ok(())
    }

    /// 更新工作区状态
    pub fn update_workspace_status(repo_path: &Path, workspace_id: &str) -> Result<WorkspaceMetadata> {
        let mut metadata = Self::load_workspace_metadata(repo_path, workspace_id)?;

        // 检查工作区路径状态
        let path_exists = metadata.workspace_path.exists();
        let is_git_repo = if path_exists {
            GitService::is_git_repository(&metadata.workspace_path)
        } else {
            false
        };

        // 更新状态
        metadata.status = if metadata.status == WorkspaceStatus::Archived {
            WorkspaceStatus::Archived  // 保持归档状态
        } else if !path_exists || !is_git_repo {
            WorkspaceStatus::Broken
        } else {
            WorkspaceStatus::Active
        };

        metadata.updated_at = chrono::Utc::now();

        // 保存更新后的元数据
        Self::save_workspace_metadata(repo_path, &metadata)?;

        Ok(metadata)
    }

    /// 访问工作区（更新最后访问时间）
    pub fn access_workspace(repo_path: &Path, workspace_id: &str) -> Result<WorkspaceMetadata> {
        let mut metadata = Self::load_workspace_metadata(repo_path, workspace_id)?;

        metadata.last_accessed_at = Some(chrono::Utc::now());
        metadata.updated_at = chrono::Utc::now();

        Self::save_workspace_metadata(repo_path, &metadata)?;

        Ok(metadata)
    }

    /// 更新工作区元数据
    pub fn update_workspace_metadata(
        repo_path: &Path,
        workspace_id: &str,
        update_fn: impl FnOnce(&mut WorkspaceMetadata) -> Result<()>,
    ) -> Result<WorkspaceMetadata> {
        let mut metadata = Self::load_workspace_metadata(repo_path, workspace_id)?;
        
        update_fn(&mut metadata)?;
        metadata.updated_at = chrono::Utc::now();
        
        Self::save_workspace_metadata(repo_path, &metadata)?;
        
        Ok(metadata)
    }

    /// 添加工作区标签
    pub fn add_workspace_tag(
        repo_path: &Path,
        workspace_id: &str,
        tag: String,
    ) -> Result<WorkspaceMetadata> {
        Self::update_workspace_metadata(repo_path, workspace_id, |metadata| {
            if !metadata.tags.contains(&tag) {
                metadata.tags.push(tag);
            }
            Ok(())
        })
    }

    /// 移除工作区标签
    pub fn remove_workspace_tag(
        repo_path: &Path,
        workspace_id: &str,
        tag: &str,
    ) -> Result<WorkspaceMetadata> {
        Self::update_workspace_metadata(repo_path, workspace_id, |metadata| {
            metadata.tags.retain(|t| t != tag);
            Ok(())
        })
    }

    /// 设置自定义字段
    pub fn set_custom_field(
        repo_path: &Path,
        workspace_id: &str,
        key: String,
        value: String,
    ) -> Result<WorkspaceMetadata> {
        Self::update_workspace_metadata(repo_path, workspace_id, |metadata| {
            metadata.custom_fields.insert(key, value);
            Ok(())
        })
    }

    /// 移除自定义字段
    pub fn remove_custom_field(
        repo_path: &Path,
        workspace_id: &str,
        key: &str,
    ) -> Result<WorkspaceMetadata> {
        Self::update_workspace_metadata(repo_path, workspace_id, |metadata| {
            metadata.custom_fields.remove(key);
            Ok(())
        })
    }

    /// 根据标签搜索工作区
    pub fn find_workspaces_by_tag(repo_path: &Path, tag: &str) -> Result<Vec<WorkspaceMetadata>> {
        let all_workspaces = Self::list_workspaces(repo_path)?;
        Ok(all_workspaces
            .into_iter()
            .filter(|ws| ws.tags.contains(&tag.to_string()))
            .collect())
    }

    /// 根据状态过滤工作区
    pub fn find_workspaces_by_status(
        repo_path: &Path,
        status: WorkspaceStatus,
    ) -> Result<Vec<WorkspaceMetadata>> {
        let all_workspaces = Self::list_workspaces(repo_path)?;
        Ok(all_workspaces
            .into_iter()
            .filter(|ws| ws.status == status)
            .collect())
    }

    /// 更新工作区索引（创建工作区概览文件）
    fn update_workspace_index(repo_path: &Path) -> Result<()> {
        let workspaces = Self::list_workspaces(repo_path)?;
        let configs_dir = RepositoryManagerService::get_configs_dir(repo_path);
        
        let index = serde_json::json!({
            "total_count": workspaces.len(),
            "active_count": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Active).count(),
            "archived_count": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Archived).count(),
            "broken_count": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Broken).count(),
            "last_updated": chrono::Utc::now(),
            "workspace_ids": workspaces.iter().map(|ws| &ws.id).collect::<Vec<_>>(),
        });

        let index_file = configs_dir.join("workspace_index.json");
        fs::write(index_file, serde_json::to_string_pretty(&index)?)?;

        Ok(())
    }

    /// 清理损坏的工作区
    pub fn cleanup_broken_workspaces(repo_path: &Path) -> Result<Vec<String>> {
        let workspaces = Self::list_workspaces(repo_path)?;
        let mut cleaned_ids = Vec::new();

        for workspace in workspaces {
            if workspace.status == WorkspaceStatus::Broken {
                if let Ok(_) = Self::delete_workspace(repo_path, &workspace.id) {
                    cleaned_ids.push(workspace.id);
                }
            }
        }

        Ok(cleaned_ids)
    }

    /// 获取工作区统计信息
    pub fn get_workspace_statistics(repo_path: &Path) -> Result<serde_json::Value> {
        let workspaces = Self::list_workspaces(repo_path)?;
        
        let mut branch_counts: HashMap<String, usize> = HashMap::new();
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        for workspace in &workspaces {
            if let Some(branch) = &workspace.branch {
                *branch_counts.entry(branch.clone()).or_insert(0) += 1;
            }
            
            for tag in &workspace.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        Ok(serde_json::json!({
            "total_count": workspaces.len(),
            "status_counts": {
                "active": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Active).count(),
                "inactive": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Inactive).count(),
                "archived": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Archived).count(),
                "broken": workspaces.iter().filter(|ws| ws.status == WorkspaceStatus::Broken).count(),
            },
            "branch_counts": branch_counts,
            "tag_counts": tag_counts,
            "oldest_workspace": workspaces.iter().min_by_key(|ws| ws.created_at).map(|ws| &ws.id),
            "newest_workspace": workspaces.iter().max_by_key(|ws| ws.created_at).map(|ws| &ws.id),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::services::{GitService, RepositoryManagerService};
    use crate::services::repository_service::AddRepositoryRequest;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // 初始化Git仓库
        GitService::init_repository(&repo_path, false).unwrap();

        // 添加到Workhorse管理
        let request = AddRepositoryRequest {
            path: repo_path.clone(),
            name: Some("test-repo".to_string()),
            default_branch: Some("main".to_string()),
            auto_fetch: false,
            auto_prune: false,
        };
        RepositoryManagerService::add_repository(request).unwrap();

        (temp_dir, repo_path)
    }

    #[test]
    fn test_create_workspace() {
        let (_temp_dir, repo_path) = setup_test_repo();

        let request = CreateWorkspaceRequest {
            name: "test-workspace".to_string(),
            repository_path: repo_path.clone(),
            branch: Some("main".to_string()),
            description: Some("Test workspace".to_string()),
            tags: vec!["test".to_string(), "feature".to_string()],
            base_path: None,
        };

        let metadata = WorkspaceManagerService::create_workspace(&repo_path, request).unwrap();

        assert_eq!(metadata.name, "test-workspace");
        assert_eq!(metadata.status, WorkspaceStatus::Active);
        assert!(metadata.workspace_path.exists());
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_archive_and_restore_workspace() {
        let (_temp_dir, repo_path) = setup_test_repo();

        // 创建工作区
        let request = CreateWorkspaceRequest {
            name: "test-workspace".to_string(),
            repository_path: repo_path.clone(),
            branch: None,
            description: None,
            tags: Vec::new(),
            base_path: None,
        };

        let metadata = WorkspaceManagerService::create_workspace(&repo_path, request).unwrap();
        let workspace_id = metadata.id.clone();

        // 归档工作区
        let archive_request = ArchiveWorkspaceRequest {
            workspace_id: workspace_id.clone(),
            keep_files: false,
            archive_reason: Some("Test archive".to_string()),
        };

        let archived_metadata = WorkspaceManagerService::archive_workspace(&repo_path, archive_request).unwrap();
        assert_eq!(archived_metadata.status, WorkspaceStatus::Archived);
        assert!(archived_metadata.archived_at.is_some());

        // 恢复工作区
        let restored_metadata = WorkspaceManagerService::restore_workspace(&repo_path, &workspace_id).unwrap();
        assert_eq!(restored_metadata.status, WorkspaceStatus::Active);
        assert!(restored_metadata.archived_at.is_none());
        assert!(restored_metadata.workspace_path.exists());
    }

    #[test]
    fn test_workspace_tags() {
        let (_temp_dir, repo_path) = setup_test_repo();

        let request = CreateWorkspaceRequest {
            name: "test-workspace".to_string(),
            repository_path: repo_path.clone(),
            branch: None,
            description: None,
            tags: vec!["initial".to_string()],
            base_path: None,
        };

        let metadata = WorkspaceManagerService::create_workspace(&repo_path, request).unwrap();
        let workspace_id = metadata.id.clone();

        // 添加标签
        let updated = WorkspaceManagerService::add_workspace_tag(&repo_path, &workspace_id, "feature".to_string()).unwrap();
        assert!(updated.tags.contains(&"feature".to_string()));

        // 移除标签
        let updated = WorkspaceManagerService::remove_workspace_tag(&repo_path, &workspace_id, "initial").unwrap();
        assert!(!updated.tags.contains(&"initial".to_string()));
    }
}