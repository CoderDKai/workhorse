use anyhow::{anyhow, Result};
use git2::{
    Branch, BranchType, Repository, Status, StatusOptions, 
    WorktreeAddOptions, WorktreePruneOptions
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub current_branch: Option<String>,
    pub is_dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub files: Vec<GitFileStatus>,
    pub repository_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
    pub is_staged: bool,
    pub is_modified: bool,
    pub is_new: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub is_locked: bool,
    pub is_prunable: bool,
}

pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        Self
    }

    /// 打开指定路径的Git仓库
    pub fn open_repository<P: AsRef<Path>>(path: P) -> Result<Repository> {
        Repository::open(path.as_ref())
            .map_err(|e| anyhow!("Failed to open repository at {:?}: {}", path.as_ref(), e))
    }

    /// 获取仓库的Git状态
    pub fn get_repository_status<P: AsRef<Path>>(repo_path: P) -> Result<GitStatus> {
        let repo = Self::open_repository(repo_path)?;
        
        // 获取当前分支
        let current_branch = Self::get_current_branch(&repo)?;
        
        // 获取文件状态
        let files = Self::get_file_statuses(&repo)?;
        
        // 检查是否有未提交的更改
        let is_dirty = !files.is_empty();
        
        // 获取ahead/behind计数（需要upstream分支）
        let (ahead, behind) = Self::get_ahead_behind_count(&repo)?;
        
        // 获取仓库状态
        let repository_state = format!("{:?}", repo.state());

        Ok(GitStatus {
            current_branch,
            is_dirty,
            ahead,
            behind,
            files,
            repository_state,
        })
    }

    /// 获取当前分支名称
    fn get_current_branch(repo: &Repository) -> Result<Option<String>> {
        match repo.head() {
            Ok(head) => {
                if let Some(name) = head.shorthand() {
                    Ok(Some(name.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => Ok(None),
            Err(e) => Err(anyhow!("Failed to get current branch: {}", e)),
        }
    }

    /// 获取文件状态列表
    fn get_file_statuses(repo: &Repository) -> Result<Vec<GitFileStatus>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);
        
        let statuses = repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let file_path = entry.path().unwrap_or("").to_string();
            let status_flags = entry.status();
            
            let status_str = Self::status_flags_to_string(status_flags);
            
            files.push(GitFileStatus {
                path: file_path,
                status: status_str,
                is_staged: status_flags.intersects(
                    Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED
                ),
                is_modified: status_flags.intersects(
                    Status::WT_MODIFIED | Status::INDEX_MODIFIED
                ),
                is_new: status_flags.intersects(
                    Status::WT_NEW | Status::INDEX_NEW
                ),
                is_deleted: status_flags.intersects(
                    Status::WT_DELETED | Status::INDEX_DELETED
                ),
            });
        }

        Ok(files)
    }

    /// 将状态标志转换为字符串
    fn status_flags_to_string(status: Status) -> String {
        let mut parts = Vec::new();
        
        if status.contains(Status::INDEX_NEW) { parts.push("A"); }
        if status.contains(Status::INDEX_MODIFIED) { parts.push("M"); }
        if status.contains(Status::INDEX_DELETED) { parts.push("D"); }
        if status.contains(Status::WT_NEW) { parts.push("??"); }
        if status.contains(Status::WT_MODIFIED) { parts.push("M"); }
        if status.contains(Status::WT_DELETED) { parts.push("D"); }
        if status.contains(Status::IGNORED) { parts.push("!"); }
        
        if parts.is_empty() {
            "clean".to_string()
        } else {
            parts.join("")
        }
    }

    /// 获取ahead/behind计数
    fn get_ahead_behind_count(repo: &Repository) -> Result<(usize, usize)> {
        let head = match repo.head() {
            Ok(head) => head,
            Err(_) => return Ok((0, 0)),
        };

        let local_oid = head.target().ok_or_else(|| anyhow!("No target for HEAD"))?;
        
        // 尝试获取upstream分支
        let branch = Branch::wrap(head);
        let upstream = match branch.upstream() {
            Ok(upstream) => upstream,
            Err(_) => return Ok((0, 0)),
        };

        let upstream_oid = upstream
            .get()
            .target()
            .ok_or_else(|| anyhow!("No target for upstream"))?;

        match repo.graph_ahead_behind(local_oid, upstream_oid) {
            Ok((ahead, behind)) => Ok((ahead, behind)),
            Err(_) => Ok((0, 0)),
        }
    }

    /// 获取所有分支列表
    pub fn get_branches<P: AsRef<Path>>(repo_path: P) -> Result<Vec<GitBranch>> {
        let repo = Self::open_repository(repo_path)?;
        let mut branches = Vec::new();

        // 获取本地分支
        let local_branches = repo.branches(Some(BranchType::Local))?;
        for branch_result in local_branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                let is_head = branch.is_head();
                let upstream = branch.upstream()
                    .ok()
                    .and_then(|up| up.name().ok().flatten().map(|s| s.to_string()));

                branches.push(GitBranch {
                    name: name.to_string(),
                    is_head,
                    is_remote: false,
                    upstream,
                });
            }
        }

        // 获取远程分支
        let remote_branches = repo.branches(Some(BranchType::Remote))?;
        for branch_result in remote_branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                branches.push(GitBranch {
                    name: name.to_string(),
                    is_head: false,
                    is_remote: true,
                    upstream: None,
                });
            }
        }

        Ok(branches)
    }

    /// 创建新的worktree
    pub fn create_worktree<P: AsRef<Path>>(
        repo_path: P,
        worktree_name: &str,
        worktree_path: P,
        branch_name: Option<&str>,
    ) -> Result<()> {
        let repo = Self::open_repository(repo_path)?;
        
        let opts = WorktreeAddOptions::new();
        
        // 如果指定了分支名，创建对应的引用
        if let Some(branch) = branch_name {
            let branch_ref = format!("refs/heads/{}", branch);
            
            // 检查分支是否存在，如果不存在则创建
            if repo.find_reference(&branch_ref).is_err() {
                let head = repo.head()?;
                let target = head.target().ok_or_else(|| anyhow!("No target for HEAD"))?;
                let commit = repo.find_commit(target)?;
                repo.branch(branch, &commit, false)?;
            }
        }

        repo.worktree(worktree_name, worktree_path.as_ref(), Some(&opts))?;
        
        Ok(())
    }

    /// 获取所有worktree列表
    pub fn list_worktrees<P: AsRef<Path>>(repo_path: P) -> Result<Vec<WorktreeInfo>> {
        let repo = Self::open_repository(repo_path)?;
        let worktree_names = repo.worktrees()?;
        let mut worktrees = Vec::new();

        for name in worktree_names.iter() {
            if let Some(name_str) = name {
                if let Ok(worktree) = repo.find_worktree(name_str) {
                    let path = worktree.path().to_path_buf();
                    let is_locked = worktree.is_locked().is_ok();
                    let is_prunable = worktree.is_prunable(None)
                        .map_err(|_| anyhow!("Failed to check if worktree is prunable"))?;

                    // 尝试获取worktree的分支
                    let branch = Self::get_worktree_branch(&repo, name_str).ok();

                    worktrees.push(WorktreeInfo {
                        name: name_str.to_string(),
                        path,
                        branch,
                        is_locked,
                        is_prunable,
                    });
                }
            }
        }

        Ok(worktrees)
    }

    /// 获取worktree的分支
    fn get_worktree_branch(repo: &Repository, worktree_name: &str) -> Result<String> {
        let worktree = repo.find_worktree(worktree_name)?;
        let worktree_path = worktree.path();
        let worktree_repo = Repository::open(worktree_path)?;
        
        let head = worktree_repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Err(anyhow!("No branch name for worktree"))
        }
    }

    /// 删除worktree
    pub fn remove_worktree<P: AsRef<Path>>(
        repo_path: P,
        worktree_name: &str,
    ) -> Result<()> {
        let repo = Self::open_repository(repo_path)?;
        let mut worktree = repo.find_worktree(worktree_name)?;
        
        let mut opts = WorktreePruneOptions::new();
        opts.valid(true);
        
        worktree.prune(Some(&mut opts))?;
        
        Ok(())
    }

    /// 切换分支
    pub fn checkout_branch<P: AsRef<Path>>(
        repo_path: P,
        branch_name: &str,
    ) -> Result<()> {
        let repo = Self::open_repository(repo_path)?;
        
        // 查找分支
        let branch_ref = format!("refs/heads/{}", branch_name);
        let reference = repo.find_reference(&branch_ref)?;
        
        // 检出分支
        let commit = reference.peel_to_commit()?;
        repo.set_head(&branch_ref)?;
        repo.checkout_tree(commit.as_object(), None)?;
        
        Ok(())
    }

    /// 创建新分支
    pub fn create_branch<P: AsRef<Path>>(
        repo_path: P,
        branch_name: &str,
        from_branch: Option<&str>,
    ) -> Result<()> {
        let repo = Self::open_repository(repo_path)?;
        
        let target_commit = if let Some(from) = from_branch {
            let from_ref = format!("refs/heads/{}", from);
            let reference = repo.find_reference(&from_ref)?;
            reference.peel_to_commit()?
        } else {
            let head = repo.head()?;
            head.peel_to_commit()?
        };

        repo.branch(branch_name, &target_commit, false)?;
        
        Ok(())
    }

    /// 设置Git全局忽略配置
    pub fn set_global_gitignore<P: AsRef<Path>>(gitignore_path: P) -> Result<()> {
        let mut config = git2::Config::open_default()?;
        let path_str = gitignore_path.as_ref().to_string_lossy();
        config.set_str("core.excludesfile", &path_str)?;
        Ok(())
    }

    /// 获取Git全局忽略配置
    pub fn get_global_gitignore() -> Result<Option<String>> {
        let config = git2::Config::open_default()?;
        match config.get_string("core.excludesfile") {
            Ok(path) => Ok(Some(path)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(anyhow!("Failed to get global gitignore: {}", e)),
        }
    }

    /// 验证路径是否为Git仓库
    pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        Repository::open(path).is_ok()
    }

    /// 初始化新的Git仓库
    pub fn init_repository<P: AsRef<Path>>(path: P, bare: bool) -> Result<Repository> {
        if bare {
            Repository::init_bare(path)
        } else {
            Repository::init(path)
        }
        .map_err(|e| anyhow!("Failed to initialize repository: {}", e))
    }

    /// 克隆远程仓库
    pub fn clone_repository<P: AsRef<Path>>(
        url: &str,
        path: P,
        progress_callback: Option<Box<dyn FnMut(git2::Progress) -> bool>>,
    ) -> Result<Repository> {
        let mut builder = git2::build::RepoBuilder::new();
        
        if let Some(mut callback) = progress_callback {
            let mut remote_callbacks = git2::RemoteCallbacks::new();
            remote_callbacks.transfer_progress(move |progress| {
                callback(progress)
            });
            
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(remote_callbacks);
            builder.fetch_options(fetch_options);
        }

        builder.clone(url, path.as_ref())
            .map_err(|e| anyhow!("Failed to clone repository: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_init_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = GitService::init_repository(repo_path, false).unwrap();
        assert!(repo_path.join(".git").exists());
    }

    #[test]
    fn test_is_git_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // 未初始化时应该返回false
        assert!(!GitService::is_git_repository(repo_path));

        // 初始化后应该返回true
        GitService::init_repository(repo_path, false).unwrap();
        assert!(GitService::is_git_repository(repo_path));
    }

    #[test]
    fn test_get_repository_status() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // 初始化仓库
        GitService::init_repository(repo_path, false).unwrap();

        // 创建一个文件
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        // 获取状态
        let status = GitService::get_repository_status(repo_path).unwrap();
        assert!(status.is_dirty);
        assert!(!status.files.is_empty());
        assert_eq!(status.files[0].path, "test.txt");
    }
}