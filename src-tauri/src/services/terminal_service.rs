use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub name: String,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub status: TerminalStatus,
    pub created_at: u64,
    pub last_activity: u64,
    pub output_history: Vec<TerminalOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalStatus {
    Active,
    Inactive,
    Closed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub timestamp: u64,
    pub content: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Stdout,
    Stderr,
    Input,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub command: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
}

pub struct TerminalInstance {
    pub session: TerminalSession,
    pub process: Option<TokioChild>,
    pub input_sender: Option<mpsc::UnboundedSender<String>>,
    pub output_receiver: Option<mpsc::UnboundedReceiver<TerminalOutput>>,
}

impl std::fmt::Debug for TerminalInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalInstance")
            .field("session", &self.session)
            .field("process", &self.process.is_some())
            .field("input_sender", &self.input_sender.is_some())
            .field("output_receiver", &self.output_receiver.is_some())
            .finish()
    }
}

#[derive(Debug)]
pub struct TerminalService {
    terminals: Arc<Mutex<HashMap<String, TerminalInstance>>>,
    max_terminals: usize,
    max_history_size: usize,
}

impl Default for TerminalService {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalService {
    pub fn new() -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            max_terminals: 10,
            max_history_size: 1000,
        }
    }

    /// 生成唯一的终端ID
    fn generate_terminal_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("term_{}", timestamp)
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// 创建新的终端会话
    pub async fn create_terminal(
        &self,
        name: Option<String>,
        working_directory: PathBuf,
        environment: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        // 检查终端数量限制
        {
            let terminals = self.terminals.lock().unwrap();
            if terminals.len() >= self.max_terminals {
                return Err("Maximum number of terminals reached".to_string());
            }
        }

        // 验证工作目录
        if !working_directory.exists() || !working_directory.is_dir() {
            return Err(format!("Invalid working directory: {:?}", working_directory));
        }

        let terminal_id = Self::generate_terminal_id();
        let session_name = name.unwrap_or_else(|| format!("Terminal {}", terminal_id));
        let env = environment.unwrap_or_default();
        let timestamp = Self::current_timestamp();

        let session = TerminalSession {
            id: terminal_id.clone(),
            name: session_name,
            working_directory,
            environment: env,
            status: TerminalStatus::Inactive,
            created_at: timestamp,
            last_activity: timestamp,
            output_history: Vec::new(),
        };

        let terminal_instance = TerminalInstance {
            session,
            process: None,
            input_sender: None,
            output_receiver: None,
        };

        {
            let mut terminals = self.terminals.lock().unwrap();
            terminals.insert(terminal_id.clone(), terminal_instance);
        }

        Ok(terminal_id)
    }

    /// 启动终端会话
    pub async fn start_terminal(&self, terminal_id: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get_mut(terminal_id)
            .ok_or("Terminal not found")?;

        if matches!(terminal.session.status, TerminalStatus::Active) {
            return Err("Terminal is already active".to_string());
        }

        // 启动shell进程
        let shell = if cfg!(windows) { "cmd" } else { "sh" };
        let mut cmd = TokioCommand::new(shell);
        
        cmd.current_dir(&terminal.session.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // 设置环境变量
        for (key, value) in &terminal.session.environment {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to start terminal process: {}", e))?;

        // 设置输入输出通道
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<String>();
        let (output_tx, output_rx) = mpsc::unbounded_channel::<TerminalOutput>();

        // 处理标准输入
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            tokio::spawn(async move {
                while let Some(input) = input_rx.recv().await {
                    if let Err(_) = stdin.write_all(input.as_bytes()).await {
                        break;
                    }
                    if let Err(_) = stdin.flush().await {
                        break;
                    }
                }
            });
        }

        // 处理标准输出
        if let Some(stdout) = child.stdout.take() {
            let output_tx_clone = output_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let output = TerminalOutput {
                        timestamp: Self::current_timestamp(),
                        content: line,
                        output_type: OutputType::Stdout,
                    };
                    if output_tx_clone.send(output).is_err() {
                        break;
                    }
                }
            });
        }

        // 处理标准错误
        if let Some(stderr) = child.stderr.take() {
            let output_tx_clone = output_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let output = TerminalOutput {
                        timestamp: Self::current_timestamp(),
                        content: line,
                        output_type: OutputType::Stderr,
                    };
                    if output_tx_clone.send(output).is_err() {
                        break;
                    }
                }
            });
        }

        terminal.process = Some(child);
        terminal.input_sender = Some(input_tx);
        terminal.output_receiver = Some(output_rx);
        terminal.session.status = TerminalStatus::Active;
        terminal.session.last_activity = Self::current_timestamp();

        // 添加系统消息
        let system_msg = TerminalOutput {
            timestamp: Self::current_timestamp(),
            content: format!("Terminal {} started", terminal_id),
            output_type: OutputType::System,
        };
        terminal.session.output_history.push(system_msg);

        Ok(())
    }

    /// 向终端发送命令
    pub async fn send_command(&self, terminal_id: &str, command: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get_mut(terminal_id)
            .ok_or("Terminal not found")?;

        if !matches!(terminal.session.status, TerminalStatus::Active) {
            return Err("Terminal is not active".to_string());
        }

        if let Some(ref input_sender) = terminal.input_sender {
            let command_with_newline = format!("{}\n", command);
            input_sender.send(command_with_newline)
                .map_err(|_| "Failed to send command to terminal")?;

            // 记录输入命令到历史
            let input_output = TerminalOutput {
                timestamp: Self::current_timestamp(),
                content: command.to_string(),
                output_type: OutputType::Input,
            };
            terminal.session.output_history.push(input_output);
            terminal.session.last_activity = Self::current_timestamp();

            // 限制历史记录大小
            if terminal.session.output_history.len() > self.max_history_size {
                terminal.session.output_history.drain(..100); // 删除最老的100条记录
            }
        } else {
            return Err("Terminal input channel not available".to_string());
        }

        Ok(())
    }

    /// 执行单个命令并等待结果
    pub async fn execute_command(&self, execution: CommandExecution) -> Result<TerminalOutput, String> {
        // 验证工作目录
        if !execution.working_directory.exists() || !execution.working_directory.is_dir() {
            return Err(format!("Invalid working directory: {:?}", execution.working_directory));
        }

        let mut cmd = TokioCommand::new(&execution.command);
        cmd.args(&execution.args)
            .current_dir(&execution.working_directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 设置环境变量
        for (key, value) in &execution.environment {
            cmd.env(key, value);
        }

        let output = cmd.output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let result_content = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        let terminal_output = TerminalOutput {
            timestamp: Self::current_timestamp(),
            content: result_content,
            output_type: if output.status.success() {
                OutputType::Stdout
            } else {
                OutputType::Stderr
            },
        };

        Ok(terminal_output)
    }

    /// 获取终端输出
    pub async fn get_terminal_output(&self, terminal_id: &str) -> Result<Vec<TerminalOutput>, String> {
        let mut terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get_mut(terminal_id)
            .ok_or("Terminal not found")?;

        let mut new_outputs = Vec::new();

        // 获取新的输出
        if let Some(ref mut output_receiver) = terminal.output_receiver {
            while let Ok(output) = output_receiver.try_recv() {
                new_outputs.push(output.clone());
                terminal.session.output_history.push(output);
            }
        }

        // 更新最后活动时间
        if !new_outputs.is_empty() {
            terminal.session.last_activity = Self::current_timestamp();
        }

        // 限制历史记录大小
        if terminal.session.output_history.len() > self.max_history_size {
            terminal.session.output_history.drain(..100);
        }

        Ok(new_outputs)
    }

    /// 获取终端历史记录
    pub fn get_terminal_history(&self, terminal_id: &str) -> Result<Vec<TerminalOutput>, String> {
        let terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get(terminal_id)
            .ok_or("Terminal not found")?;

        Ok(terminal.session.output_history.clone())
    }

    /// 关闭终端
    pub async fn close_terminal(&self, terminal_id: &str) -> Result<(), String> {
        let mut process_to_kill = None;
        
        // 获取需要关闭的进程
        {
            let mut terminals = self.terminals.lock().unwrap();
            let terminal = terminals
                .get_mut(terminal_id)
                .ok_or("Terminal not found")?;

            // 取出进程以便在锁外关闭
            process_to_kill = terminal.process.take();
            
            terminal.session.status = TerminalStatus::Closed;
            terminal.session.last_activity = Self::current_timestamp();

            // 添加关闭消息
            let close_msg = TerminalOutput {
                timestamp: Self::current_timestamp(),
                content: format!("Terminal {} closed", terminal_id),
                output_type: OutputType::System,
            };
            terminal.session.output_history.push(close_msg);
        }
        
        // 在锁外关闭进程
        if let Some(mut process) = process_to_kill {
            let _ = process.kill().await;
        }

        Ok(())
    }

    /// 获取终端会话信息
    pub fn get_terminal_session(&self, terminal_id: &str) -> Option<TerminalSession> {
        let terminals = self.terminals.lock().unwrap();
        terminals.get(terminal_id).map(|t| t.session.clone())
    }

    /// 获取所有终端会话
    pub fn get_all_terminals(&self) -> Vec<TerminalSession> {
        let terminals = self.terminals.lock().unwrap();
        terminals.values().map(|t| t.session.clone()).collect()
    }

    /// 清理已关闭的终端
    pub fn cleanup_closed_terminals(&self) {
        let mut terminals = self.terminals.lock().unwrap();
        terminals.retain(|_, terminal| {
            !matches!(terminal.session.status, TerminalStatus::Closed)
        });
    }

    /// 设置终端名称
    pub fn set_terminal_name(&self, terminal_id: &str, name: String) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get_mut(terminal_id)
            .ok_or("Terminal not found")?;

        terminal.session.name = name;
        terminal.session.last_activity = Self::current_timestamp();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_create_terminal() {
        let service = TerminalService::new();
        let working_dir = env::temp_dir();

        let terminal_id = service
            .create_terminal(Some("Test Terminal".to_string()), working_dir, None)
            .await
            .unwrap();

        let session = service.get_terminal_session(&terminal_id).unwrap();
        assert_eq!(session.name, "Test Terminal");
        assert_eq!(session.status, TerminalStatus::Inactive);
    }

    #[tokio::test]
    async fn test_execute_command() {
        let service = TerminalService::new();
        let working_dir = env::temp_dir();

        let execution = CommandExecution {
            command: "echo".to_string(),
            args: vec!["Hello World".to_string()],
            working_directory: working_dir,
            environment: HashMap::new(),
        };

        let output = service.execute_command(execution).await.unwrap();
        assert!(output.content.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_terminal_lifecycle() {
        let service = TerminalService::new();
        let working_dir = env::temp_dir();

        // 创建终端
        let terminal_id = service
            .create_terminal(None, working_dir, None)
            .await
            .unwrap();

        // 启动终端
        service.start_terminal(&terminal_id).await.unwrap();

        // 检查状态
        let session = service.get_terminal_session(&terminal_id).unwrap();
        assert_eq!(session.status, TerminalStatus::Active);

        // 关闭终端
        service.close_terminal(&terminal_id).await.unwrap();

        // 检查关闭状态
        let session = service.get_terminal_session(&terminal_id).unwrap();
        assert_eq!(session.status, TerminalStatus::Closed);
    }
}