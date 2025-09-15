use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use crate::services::GitService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub name: String,
    pub path: PathBuf,
    pub default_branch: Option<String>,
    pub workhorse_dir: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub auto_fetch: bool,
    pub auto_prune: bool,
    pub scripts: Vec<RepositoryScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryScript {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub working_directory: Option<PathBuf>,
    pub env_vars: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryValidationResult {
    pub is_valid: bool,
    pub is_git_repo: bool,
    pub has_workhorse_dir: bool,
    pub path_exists: bool,
    pub error_message: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRepositoryRequest {
    pub path: PathBuf,
    pub name: Option<String>,
    pub default_branch: Option<String>,
    pub auto_fetch: bool,
    pub auto_prune: bool,
}

pub struct RepositoryManagerService;

impl RepositoryManagerService {
    pub fn new() -> Self {
        Self
    }

    /// 验证仓库路径是否有效
    pub fn validate_repository<P: AsRef<Path>>(repo_path: P) -> Result<RepositoryValidationResult> {
        let path = repo_path.as_ref();
        let mut result = RepositoryValidationResult {
            is_valid: false,
            is_git_repo: false,
            has_workhorse_dir: false,
            path_exists: false,
            error_message: None,
            warnings: Vec::new(),
        };

        // 检查路径是否存在
        if !path.exists() {
            result.error_message = Some("指定的路径不存在".to_string());
            return Ok(result);
        }

        result.path_exists = true;

        // 检查是否为目录
        if !path.is_dir() {
            result.error_message = Some("指定的路径不是目录".to_string());
            return Ok(result);
        }

        // 检查是否为Git仓库
        result.is_git_repo = GitService::is_git_repository(path);
        if !result.is_git_repo {
            result.error_message = Some("指定的目录不是Git仓库".to_string());
            return Ok(result);
        }

        // 检查是否已有.workhorse目录
        let workhorse_dir = path.join(".workhorse");
        result.has_workhorse_dir = workhorse_dir.exists();

        if result.has_workhorse_dir {
            result.warnings.push("仓库已经包含.workhorse目录，可能已经被管理".to_string());
        }

        // 检查是否有写权限
        if let Err(_) = fs::metadata(path) {
            result.warnings.push("无法读取目录权限信息".to_string());
        }

        result.is_valid = result.path_exists && result.is_git_repo;

        Ok(result)
    }

    /// 创建.workhorse目录结构
    pub fn create_workhorse_directory<P: AsRef<Path>>(repo_path: P) -> Result<PathBuf> {
        let repo_path = repo_path.as_ref();
        let workhorse_dir = repo_path.join(".workhorse");

        // 如果目录已存在，检查是否为有效的workhorse目录
        if workhorse_dir.exists() {
            if !workhorse_dir.is_dir() {
                return Err(anyhow!(".workhorse 已存在但不是目录"));
            }
            // 验证现有目录结构
            Self::ensure_workhorse_structure(&workhorse_dir)?;
            return Ok(workhorse_dir);
        }

        // 创建主目录
        fs::create_dir_all(&workhorse_dir)
            .map_err(|e| anyhow!("创建.workhorse目录失败: {}", e))?;

        // 创建子目录结构
        Self::ensure_workhorse_structure(&workhorse_dir)?;

        Ok(workhorse_dir)
    }

    /// 确保.workhorse目录结构完整
    fn ensure_workhorse_structure(workhorse_dir: &Path) -> Result<()> {
        let subdirs = [
            "workspaces",     // 工作区元数据
            "configs",        // 配置文件
            "scripts",        // 脚本文件
            "logs",          // 日志文件
            "temp",          // 临时文件
        ];

        for subdir in &subdirs {
            let dir_path = workhorse_dir.join(subdir);
            if !dir_path.exists() {
                fs::create_dir_all(&dir_path)
                    .map_err(|e| anyhow!("创建{}目录失败: {}", subdir, e))?;
            }
        }

        // 创建默认配置文件
        let config_file = workhorse_dir.join("configs").join("repository.json");
        if !config_file.exists() {
            let default_config = serde_json::json!({
                "version": "1.0",
                "created_at": chrono::Utc::now(),
                "auto_fetch": false,
                "auto_prune": false,
                "scripts": []
            });

            fs::write(&config_file, serde_json::to_string_pretty(&default_config)?)
                .map_err(|e| anyhow!("创建默认配置文件失败: {}", e))?;
        }

        // 创建.gitignore文件（如果不存在）
        let gitignore_file = workhorse_dir.join(".gitignore");
        if !gitignore_file.exists() {
            let gitignore_content = "# Workhorse temporary files\ntemp/\nlogs/*.log\n";
            fs::write(&gitignore_file, gitignore_content)
                .map_err(|e| anyhow!("创建.gitignore文件失败: {}", e))?;
        }

        Ok(())
    }

    /// 添加新仓库到管理
    pub fn add_repository(request: AddRepositoryRequest) -> Result<RepositoryConfig> {
        // 验证仓库
        let validation = Self::validate_repository(&request.path)?;
        if !validation.is_valid {
            return Err(anyhow!("仓库验证失败: {}", 
                validation.error_message.unwrap_or_else(|| "未知错误".to_string())));
        }

        // 创建.workhorse目录
        let workhorse_dir = Self::create_workhorse_directory(&request.path)?;

        // 生成仓库名称（如果未提供）
        let name = request.name.unwrap_or_else(|| {
            request.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repository")
                .to_string()
        });

        // 获取默认分支（如果未提供）
        let default_branch = if let Some(branch) = request.default_branch {
            Some(branch)
        } else {
            // 尝试从Git仓库获取当前分支
            GitService::get_repository_status(&request.path)
                .ok()
                .and_then(|status| status.current_branch)
        };

        let now = chrono::Utc::now();
        let config = RepositoryConfig {
            name,
            path: request.path.clone(),
            default_branch,
            workhorse_dir,
            created_at: now,
            updated_at: now,
            auto_fetch: request.auto_fetch,
            auto_prune: request.auto_prune,
            scripts: Vec::new(),
        };

        // 保存配置到.workhorse/configs/repository.json
        Self::save_repository_config(&config)?;

        Ok(config)
    }

    /// 加载仓库配置
    pub fn load_repository_config<P: AsRef<Path>>(repo_path: P) -> Result<RepositoryConfig> {
        let repo_path = repo_path.as_ref();
        let config_file = repo_path.join(".workhorse").join("configs").join("repository.json");

        if !config_file.exists() {
            return Err(anyhow!("仓库配置文件不存在"));
        }

        let config_content = fs::read_to_string(&config_file)
            .map_err(|e| anyhow!("读取配置文件失败: {}", e))?;

        let mut config: RepositoryConfig = serde_json::from_str(&config_content)
            .map_err(|e| anyhow!("解析配置文件失败: {}", e))?;

        // 更新路径信息（可能配置文件被移动了）
        config.path = repo_path.to_path_buf();
        config.workhorse_dir = repo_path.join(".workhorse");

        Ok(config)
    }

    /// 保存仓库配置
    pub fn save_repository_config(config: &RepositoryConfig) -> Result<()> {
        let config_file = config.workhorse_dir.join("configs").join("repository.json");
        
        // 确保目录存在
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("创建配置目录失败: {}", e))?;
        }

        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| anyhow!("序列化配置失败: {}", e))?;

        fs::write(&config_file, config_json)
            .map_err(|e| anyhow!("写入配置文件失败: {}", e))?;

        Ok(())
    }

    /// 更新仓库配置
    pub fn update_repository_config<P: AsRef<Path>>(
        repo_path: P,
        update_fn: impl FnOnce(&mut RepositoryConfig) -> Result<()>,
    ) -> Result<RepositoryConfig> {
        let mut config = Self::load_repository_config(repo_path)?;
        
        update_fn(&mut config)?;
        config.updated_at = chrono::Utc::now();
        
        Self::save_repository_config(&config)?;
        
        Ok(config)
    }

    /// 添加脚本到仓库配置
    pub fn add_script<P: AsRef<Path>>(
        repo_path: P,
        script: RepositoryScript,
    ) -> Result<RepositoryConfig> {
        Self::update_repository_config(repo_path, |config| {
            // 检查脚本名称是否已存在
            if config.scripts.iter().any(|s| s.name == script.name) {
                return Err(anyhow!("脚本名称 '{}' 已存在", script.name));
            }
            
            config.scripts.push(script);
            Ok(())
        })
    }

    /// 删除脚本
    pub fn remove_script<P: AsRef<Path>>(
        repo_path: P,
        script_name: &str,
    ) -> Result<RepositoryConfig> {
        Self::update_repository_config(repo_path, |config| {
            let initial_len = config.scripts.len();
            config.scripts.retain(|s| s.name != script_name);
            
            if config.scripts.len() == initial_len {
                return Err(anyhow!("脚本 '{}' 不存在", script_name));
            }
            
            Ok(())
        })
    }

    /// 获取脚本列表
    pub fn get_scripts<P: AsRef<Path>>(repo_path: P) -> Result<Vec<RepositoryScript>> {
        let config = Self::load_repository_config(repo_path)?;
        Ok(config.scripts)
    }

    /// 检查仓库是否被Workhorse管理
    pub fn is_managed_repository<P: AsRef<Path>>(repo_path: P) -> bool {
        let workhorse_dir = repo_path.as_ref().join(".workhorse");
        let config_file = workhorse_dir.join("configs").join("repository.json");
        
        workhorse_dir.exists() && config_file.exists()
    }

    /// 移除仓库管理（删除.workhorse目录）
    pub fn remove_repository_management<P: AsRef<Path>>(repo_path: P) -> Result<()> {
        let workhorse_dir = repo_path.as_ref().join(".workhorse");
        
        if workhorse_dir.exists() {
            fs::remove_dir_all(&workhorse_dir)
                .map_err(|e| anyhow!("删除.workhorse目录失败: {}", e))?;
        }
        
        Ok(())
    }

    /// 获取工作区目录路径
    pub fn get_workspaces_dir<P: AsRef<Path>>(repo_path: P) -> PathBuf {
        repo_path.as_ref().join(".workhorse").join("workspaces")
    }

    /// 获取配置目录路径
    pub fn get_configs_dir<P: AsRef<Path>>(repo_path: P) -> PathBuf {
        repo_path.as_ref().join(".workhorse").join("configs")
    }

    /// 获取脚本目录路径
    pub fn get_scripts_dir<P: AsRef<Path>>(repo_path: P) -> PathBuf {
        repo_path.as_ref().join(".workhorse").join("scripts")
    }

    /// 获取日志目录路径
    pub fn get_logs_dir<P: AsRef<Path>>(repo_path: P) -> PathBuf {
        repo_path.as_ref().join(".workhorse").join("logs")
    }

    /// 获取临时目录路径
    pub fn get_temp_dir<P: AsRef<Path>>(repo_path: P) -> PathBuf {
        repo_path.as_ref().join(".workhorse").join("temp")
    }

    /// 清理临时文件
    pub fn cleanup_temp_files<P: AsRef<Path>>(repo_path: P) -> Result<()> {
        let temp_dir = Self::get_temp_dir(repo_path);
        
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)
                .map_err(|e| anyhow!("清理临时目录失败: {}", e))?;
            
            fs::create_dir_all(&temp_dir)
                .map_err(|e| anyhow!("重建临时目录失败: {}", e))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::services::GitService;

    #[test]
    fn test_validate_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // 测试无效路径
        let invalid_path = repo_path.join("nonexistent");
        let result = RepositoryManagerService::validate_repository(&invalid_path).unwrap();
        assert!(!result.is_valid);
        assert!(!result.path_exists);

        // 初始化Git仓库
        GitService::init_repository(repo_path, false).unwrap();

        // 测试有效仓库
        let result = RepositoryManagerService::validate_repository(repo_path).unwrap();
        assert!(result.is_valid);
        assert!(result.path_exists);
        assert!(result.is_git_repo);
        assert!(!result.has_workhorse_dir);
    }

    #[test]
    fn test_create_workhorse_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        GitService::init_repository(repo_path, false).unwrap();

        let workhorse_dir = RepositoryManagerService::create_workhorse_directory(repo_path).unwrap();
        
        assert!(workhorse_dir.exists());
        assert!(workhorse_dir.join("workspaces").exists());
        assert!(workhorse_dir.join("configs").exists());
        assert!(workhorse_dir.join("scripts").exists());
        assert!(workhorse_dir.join("logs").exists());
        assert!(workhorse_dir.join("temp").exists());
        assert!(workhorse_dir.join("configs").join("repository.json").exists());
        assert!(workhorse_dir.join(".gitignore").exists());
    }

    #[test]
    fn test_add_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        GitService::init_repository(repo_path, false).unwrap();

        let request = AddRepositoryRequest {
            path: repo_path.to_path_buf(),
            name: Some("test-repo".to_string()),
            default_branch: Some("main".to_string()),
            auto_fetch: true,
            auto_prune: false,
        };

        let config = RepositoryManagerService::add_repository(request).unwrap();
        
        assert_eq!(config.name, "test-repo");
        assert_eq!(config.path, repo_path);
        assert_eq!(config.default_branch, Some("main".to_string()));
        assert!(config.auto_fetch);
        assert!(!config.auto_prune);
        assert!(config.scripts.is_empty());
    }

    #[test]
    fn test_is_managed_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        GitService::init_repository(repo_path, false).unwrap();

        // 未管理的仓库
        assert!(!RepositoryManagerService::is_managed_repository(repo_path));

        // 添加管理
        let request = AddRepositoryRequest {
            path: repo_path.to_path_buf(),
            name: None,
            default_branch: None,
            auto_fetch: false,
            auto_prune: false,
        };

        RepositoryManagerService::add_repository(request).unwrap();

        // 现在应该被管理
        assert!(RepositoryManagerService::is_managed_repository(repo_path));
    }
}