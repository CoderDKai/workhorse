// 重新导出数据库相关类型
export type {
  Repository,
  Workspace,
  CreateRepositoryRequest,
  CreateWorkspaceRequest,
  UpdateRepositoryRequest,
  ApiResponse
} from './database';

// 通用类型

// Git 服务相关类型
export interface GitStatus {
  is_clean: boolean;
  modified_files: string[];
  untracked_files: string[];
  staged_files: string[];
  deleted_files: string[];
  renamed_files: string[];
  conflicted_files: string[];
  branch_name: string | null;
  has_uncommitted_changes: boolean;
}

export interface GitBranch {
  name: string;
  is_head: boolean;
  is_remote: boolean;
  upstream: string | null;
}

export interface WorktreeInfo {
  path: string;
  branch: string | null;
  is_main: boolean;
  is_detached: boolean;
  head_commit: string | null;
}

// 仓库管理服务相关类型
export interface RepositoryValidationResult {
  is_valid: boolean;
  is_git_repo: boolean;
  has_remote: boolean;
  errors: string[];
  warnings: string[];
}

export interface RepositoryConfig {
  repository_path: string;
  default_branch: string;
  scripts: RepositoryScript[];
  ignored_patterns: string[];
  created_at: string;
  updated_at: string;
}

export interface RepositoryScript {
  name: string;
  content: string;
  description: string | null;
  tags: string[];
  working_directory: string | null;
  environment: Record<string, string>;
  timeout_seconds: number | null;
  created_at: string;
  updated_at: string;
}

export interface AddRepositoryRequest {
  path: string;
  default_branch: string;
  setup_workhorse_dir: boolean;
}

// 工作区管理服务相关类型
export interface WorkspaceMetadata {
  id: string;
  name: string;
  description: string | null;
  path: string;
  branch_name: string;
  status: WorkspaceStatus;
  tags: string[];
  custom_fields: Record<string, string>;
  created_at: string;
  updated_at: string;
  last_accessed_at: string | null;
  archived_at: string | null;
}

export interface WorkspaceInfo {
  metadata: WorkspaceMetadata;
  git_status: GitStatus | null;
  disk_usage: number | null;
  has_uncommitted_changes: boolean;
  is_directory_exists: boolean;
}

export interface CreateManagedWorkspaceRequest {
  name: string;
  description: string | null;
  branch_name: string;
  create_branch_if_not_exists: boolean;
  tags: string[];
  custom_fields: Record<string, string>;
}

export interface ArchiveWorkspaceRequest {
  workspace_id: string;
  reason: string | null;
  cleanup_files: boolean;
}

export enum WorkspaceStatus {
  Active = "Active",
  Archived = "Archived",
  Broken = "Broken",
  Initializing = "Initializing",
}

// 脚本执行相关类型
export interface ScriptExecution {
  id: string;
  script_content: string;
  working_directory: string;
  environment: Record<string, string>;
  status: ExecutionStatus;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  exit_code: number | null;
  stdout: string;
  stderr: string;
}

export interface ScriptExecutionResult {
  execution_id: string;
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

export enum ExecutionStatus {
  Pending = "Pending",
  Running = "Running",
  Completed = "Completed",
  Failed = "Failed",
  Cancelled = "Cancelled",
}

// 终端服务相关类型
export interface TerminalSession {
  id: string;
  name: string | null;
  working_directory: string;
  environment: Record<string, string>;
  status: TerminalStatus;
  created_at: string;
  started_at: string | null;
  last_command_at: string | null;
  command_count: number;
}

export interface TerminalOutput {
  timestamp: string;
  output_type: OutputType;
  content: string;
  command_id: string | null;
}

export interface CommandExecution {
  command: string;
  args: string[];
  working_directory: string;
  environment: Record<string, string>;
}

export enum TerminalStatus {
  Created = "Created",
  Running = "Running",
  Closed = "Closed",
  Error = "Error",
}

export enum OutputType {
  Stdout = "Stdout",
  Stderr = "Stderr",
  Command = "Command",
  SystemMessage = "SystemMessage",
}