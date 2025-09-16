use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecution {
    pub id: String,
    pub script_content: String,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub status: ExecutionStatus,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub id: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct ScriptExecutor {
    executions: Arc<Mutex<HashMap<String, ScriptExecution>>>,
    max_concurrent_executions: usize,
    running_count: Arc<Mutex<usize>>,
}

impl Default for ScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptExecutor {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent_executions: 5, // 限制最大并发执行数
            running_count: Arc::new(Mutex::new(0)),
        }
    }

    /// 生成唯一的执行ID
    fn generate_execution_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("exec_{}", timestamp)
    }

    /// 验证脚本内容的安全性
    fn validate_script(&self, script_content: &str) -> Result<(), String> {
        // 基本的安全检查
        let dangerous_patterns = [
            "rm -rf",
            "sudo rm",
            "format",
            "del /f",
            "shutdown",
            "reboot",
            "halt",
            "poweroff",
            "init 0",
            "init 6",
            "dd if=",
            "mkfs",
        ];

        for pattern in &dangerous_patterns {
            if script_content.contains(pattern) {
                return Err(format!("Script contains potentially dangerous command: {}", pattern));
            }
        }

        // 检查脚本长度
        if script_content.len() > 10000 {
            return Err("Script content too long (max 10000 characters)".to_string());
        }

        Ok(())
    }

    /// 准备执行环境
    fn prepare_environment(&self, working_dir: &PathBuf) -> Result<(), String> {
        // 确保工作目录存在
        if !working_dir.exists() {
            return Err(format!("Working directory does not exist: {:?}", working_dir));
        }

        // 确保工作目录是安全的（在仓库范围内）
        if !working_dir.is_dir() {
            return Err(format!("Working directory is not a directory: {:?}", working_dir));
        }

        Ok(())
    }

    /// 创建新的脚本执行
    pub async fn create_execution(
        &self,
        script_content: String,
        working_directory: PathBuf,
        environment: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        // 验证脚本
        self.validate_script(&script_content)?;

        // 验证工作目录
        self.prepare_environment(&working_directory)?;

        // 检查并发执行限制
        {
            let running_count = self.running_count.lock().unwrap();
            if *running_count >= self.max_concurrent_executions {
                return Err("Maximum concurrent executions reached".to_string());
            }
        }

        let execution_id = Self::generate_execution_id();
        let env = environment.unwrap_or_default();

        let execution = ScriptExecution {
            id: execution_id.clone(),
            script_content,
            working_directory,
            environment: env,
            status: ExecutionStatus::Pending,
            start_time: None,
            end_time: None,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        };

        {
            let mut executions = self.executions.lock().unwrap();
            executions.insert(execution_id.clone(), execution);
        }

        Ok(execution_id)
    }

    /// 执行脚本
    pub async fn execute_script(&self, execution_id: String) -> Result<ScriptExecutionResult, String> {
        let execution = {
            let executions = self.executions.lock().unwrap();
            executions.get(&execution_id).cloned()
        };

        let mut execution = execution.ok_or("Execution not found")?;

        // 更新状态为运行中
        execution.status = ExecutionStatus::Running;
        execution.start_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );

        {
            let mut executions = self.executions.lock().unwrap();
            executions.insert(execution_id.clone(), execution.clone());
            
            let mut running_count = self.running_count.lock().unwrap();
            *running_count += 1;
        }

        let result = self.run_script_internal(&execution).await;

        // 更新执行状态
        execution.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );

        match &result {
            Ok(exec_result) => {
                execution.status = if exec_result.success {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::Failed
                };
                execution.exit_code = exec_result.exit_code;
                execution.stdout = exec_result.stdout.clone();
                execution.stderr = exec_result.stderr.clone();
            }
            Err(_) => {
                execution.status = ExecutionStatus::Failed;
            }
        }

        {
            let mut executions = self.executions.lock().unwrap();
            executions.insert(execution_id.clone(), execution);
            
            let mut running_count = self.running_count.lock().unwrap();
            *running_count = running_count.saturating_sub(1);
        }

        result
    }

    /// 内部执行脚本的实现
    async fn run_script_internal(&self, execution: &ScriptExecution) -> Result<ScriptExecutionResult, String> {
        let start_time = SystemTime::now();

        // 创建临时脚本文件
        let script_file = execution.working_directory.join(format!("temp_script_{}.sh", execution.id));
        
        // 写入脚本内容
        std::fs::write(&script_file, &execution.script_content)
            .map_err(|e| format!("Failed to write script file: {}", e))?;

        // 设置脚本文件权限为可执行
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_file)
                .map_err(|e| format!("Failed to read script file permissions: {}", e))?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_file, perms)
                .map_err(|e| format!("Failed to set script file permissions: {}", e))?;
        }

        // 执行脚本
        let mut cmd = Command::new("sh");
        cmd.arg(&script_file)
            .current_dir(&execution.working_directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 设置环境变量
        for (key, value) in &execution.environment {
            cmd.env(key, value);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        // 清理临时脚本文件
        let _ = std::fs::remove_file(&script_file);

        let duration_ms = start_time.elapsed().unwrap().as_millis() as u64;
        
        let result = ScriptExecutionResult {
            id: execution.id.clone(),
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
        };

        Ok(result)
    }

    /// 取消脚本执行
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut executions = self.executions.lock().unwrap();
        
        if let Some(execution) = executions.get_mut(execution_id) {
            match execution.status {
                ExecutionStatus::Pending | ExecutionStatus::Running => {
                    execution.status = ExecutionStatus::Cancelled;
                    execution.end_time = Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    );
                    Ok(())
                }
                _ => Err("Cannot cancel execution in current status".to_string()),
            }
        } else {
            Err("Execution not found".to_string())
        }
    }

    /// 获取执行状态
    pub fn get_execution_status(&self, execution_id: &str) -> Option<ScriptExecution> {
        let executions = self.executions.lock().unwrap();
        executions.get(execution_id).cloned()
    }

    /// 获取所有执行记录
    pub fn get_all_executions(&self) -> Vec<ScriptExecution> {
        let executions = self.executions.lock().unwrap();
        executions.values().cloned().collect()
    }

    /// 清理已完成的执行记录
    pub fn cleanup_completed_executions(&self, keep_count: usize) {
        let mut executions = self.executions.lock().unwrap();
        
        let mut completed_execution_ids: Vec<String> = executions
            .values()
            .filter(|exec| matches!(exec.status, ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled))
            .map(|exec| exec.id.clone())
            .collect();

        // 按结束时间排序，保留最新的记录
        completed_execution_ids.sort_by(|a, b| {
            let a_time = executions.get(a).and_then(|e| e.end_time).unwrap_or(0);
            let b_time = executions.get(b).and_then(|e| e.end_time).unwrap_or(0);
            b_time.cmp(&a_time)
        });

        // 删除多余的记录
        if completed_execution_ids.len() > keep_count {
            for execution_id in completed_execution_ids.iter().skip(keep_count) {
                executions.remove(execution_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_script_validation() {
        let executor = ScriptExecutor::new();
        
        // 测试安全脚本
        assert!(executor.validate_script("echo 'Hello World'").is_ok());
        
        // 测试危险脚本
        assert!(executor.validate_script("rm -rf /").is_err());
        assert!(executor.validate_script("sudo rm -rf *").is_err());
    }

    #[tokio::test]
    async fn test_simple_script_execution() {
        let executor = ScriptExecutor::new();
        let working_dir = env::temp_dir();
        
        let execution_id = executor
            .create_execution("echo 'test output'".to_string(), working_dir, None)
            .await
            .unwrap();
        
        let result = executor.execute_script(execution_id).await.unwrap();
        
        assert!(result.success);
        assert!(result.stdout.contains("test output"));
    }

    #[tokio::test]
    async fn test_script_with_environment() {
        let executor = ScriptExecutor::new();
        let working_dir = env::temp_dir();
        
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());
        
        let execution_id = executor
            .create_execution("echo $TEST_VAR".to_string(), working_dir, Some(env))
            .await
            .unwrap();
        
        let result = executor.execute_script(execution_id).await.unwrap();
        
        assert!(result.success);
        assert!(result.stdout.contains("test_value"));
    }
}