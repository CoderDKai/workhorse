import { invoke } from '@tauri-apps/api/core';
import type {
  ApiResponse,
  Repository,
  Workspace,
  CreateRepositoryRequest,
  CreateWorkspaceRequest,
  UpdateRepositoryRequest,
  GitStatus,
  GitBranch,
  WorktreeInfo,
  RepositoryValidationResult,
  RepositoryConfig,
  RepositoryScript,
  AddRepositoryRequest,
  WorkspaceMetadata,
  WorkspaceInfo,
  CreateManagedWorkspaceRequest,
  ArchiveWorkspaceRequest,
  WorkspaceStatus,
  ScriptExecution,
  ScriptExecutionResult,
  TerminalSession,
  TerminalOutput,
} from '../types/api';

export class TauriAPIClient {
  // 基础命令
  static async greet(name: string): Promise<string> {
    return await invoke('greet', { name });
  }

  static async databaseHealthCheck(): Promise<ApiResponse<boolean>> {
    return await invoke('database_health_check');
  }

  // 数据库仓库管理
  static async createRepository(request: CreateRepositoryRequest): Promise<ApiResponse<Repository>> {
    return await invoke('create_repository', { request });
  }

  static async getRepositories(): Promise<ApiResponse<Repository[]>> {
    return await invoke('get_repositories');
  }

  static async getRepositoryById(id: string): Promise<ApiResponse<Repository | null>> {
    return await invoke('get_repository_by_id', { id });
  }

  static async updateRepository(id: string, request: UpdateRepositoryRequest): Promise<ApiResponse<Repository | null>> {
    return await invoke('update_repository', { id, request });
  }

  static async deleteRepository(id: string): Promise<ApiResponse<boolean>> {
    return await invoke('delete_repository', { id });
  }

  // 数据库工作区管理
  static async createWorkspace(request: CreateWorkspaceRequest, path: string): Promise<ApiResponse<Workspace>> {
    return await invoke('create_workspace', { request, path });
  }

  static async getWorkspacesByRepository(repositoryId: string): Promise<ApiResponse<Workspace[]>> {
    return await invoke('get_workspaces_by_repository', { repositoryId });
  }

  static async archiveWorkspace(id: string): Promise<ApiResponse<Workspace | null>> {
    return await invoke('archive_workspace', { id });
  }

  static async restoreWorkspace(id: string): Promise<ApiResponse<Workspace | null>> {
    return await invoke('restore_workspace', { id });
  }

  static async deleteWorkspace(id: string): Promise<ApiResponse<boolean>> {
    return await invoke('delete_workspace', { id });
  }

  // Git 操作
  static async getGitStatus(repoPath: string): Promise<ApiResponse<GitStatus>> {
    return await invoke('get_git_status', { repoPath });
  }

  static async getGitBranches(repoPath: string): Promise<ApiResponse<GitBranch[]>> {
    return await invoke('get_git_branches', { repoPath });
  }

  static async createGitWorktree(
    repoPath: string,
    worktreeName: string,
    worktreePath: string,
    branchName?: string
  ): Promise<ApiResponse<boolean>> {
    return await invoke('create_git_worktree', {
      repoPath,
      worktreeName,
      worktreePath,
      branchName: branchName || null,
    });
  }

  static async listGitWorktrees(repoPath: string): Promise<ApiResponse<WorktreeInfo[]>> {
    return await invoke('list_git_worktrees', { repoPath });
  }

  static async removeGitWorktree(repoPath: string, worktreeName: string): Promise<ApiResponse<boolean>> {
    return await invoke('remove_git_worktree', { repoPath, worktreeName });
  }

  static async checkoutGitBranch(repoPath: string, branchName: string): Promise<ApiResponse<boolean>> {
    return await invoke('checkout_git_branch', { repoPath, branchName });
  }

  static async createGitBranch(repoPath: string, branchName: string, fromBranch?: string): Promise<ApiResponse<boolean>> {
    return await invoke('create_git_branch', {
      repoPath,
      branchName,
      fromBranch: fromBranch || null,
    });
  }

  static async isGitRepository(path: string): Promise<ApiResponse<boolean>> {
    return await invoke('is_git_repository', { path });
  }

  static async initGitRepository(path: string, bare: boolean = false): Promise<ApiResponse<boolean>> {
    return await invoke('init_git_repository', { path, bare });
  }

  static async cloneGitRepository(url: string, path: string): Promise<ApiResponse<boolean>> {
    return await invoke('clone_git_repository', { url, path });
  }

  static async setGlobalGitignore(gitignorePath: string): Promise<ApiResponse<boolean>> {
    return await invoke('set_global_gitignore', { gitignorePath });
  }

  static async getGlobalGitignore(): Promise<ApiResponse<string | null>> {
    return await invoke('get_global_gitignore');
  }

  // 仓库管理
  static async validateRepository(repoPath: string): Promise<ApiResponse<RepositoryValidationResult>> {
    return await invoke('validate_repository', { repoPath });
  }

  static async addRepositoryManagement(request: AddRepositoryRequest): Promise<ApiResponse<RepositoryConfig>> {
    return await invoke('add_repository_management', { request });
  }

  static async loadRepositoryConfig(repoPath: string): Promise<ApiResponse<RepositoryConfig>> {
    return await invoke('load_repository_config', { repoPath });
  }

  static async isManagedRepository(repoPath: string): Promise<ApiResponse<boolean>> {
    return await invoke('is_managed_repository', { repoPath });
  }

  static async removeRepositoryManagement(repoPath: string): Promise<ApiResponse<boolean>> {
    return await invoke('remove_repository_management', { repoPath });
  }

  static async addRepositoryScript(repoPath: string, script: RepositoryScript): Promise<ApiResponse<RepositoryConfig>> {
    return await invoke('add_repository_script', { repoPath, script });
  }

  static async removeRepositoryScript(repoPath: string, scriptName: string): Promise<ApiResponse<RepositoryConfig>> {
    return await invoke('remove_repository_script', { repoPath, scriptName });
  }

  static async getRepositoryScripts(repoPath: string): Promise<ApiResponse<RepositoryScript[]>> {
    return await invoke('get_repository_scripts', { repoPath });
  }

  static async createWorkhorseDirectory(repoPath: string): Promise<ApiResponse<string>> {
    return await invoke('create_workhorse_directory', { repoPath });
  }

  static async cleanupRepositoryTempFiles(repoPath: string): Promise<ApiResponse<boolean>> {
    return await invoke('cleanup_repository_temp_files', { repoPath });
  }

  static async getRepositoryDirectories(repoPath: string): Promise<ApiResponse<Record<string, string>>> {
    return await invoke('get_repository_directories', { repoPath });
  }

  // 工作区管理
  static async createManagedWorkspace(
    repoPath: string,
    request: CreateManagedWorkspaceRequest
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('create_managed_workspace', { repoPath, request });
  }

  static async listManagedWorkspaces(repoPath: string): Promise<ApiResponse<WorkspaceMetadata[]>> {
    return await invoke('list_managed_workspaces', { repoPath });
  }

  static async getManagedWorkspaceInfo(repoPath: string, workspaceId: string): Promise<ApiResponse<WorkspaceInfo>> {
    return await invoke('get_managed_workspace_info', { repoPath, workspaceId });
  }

  static async archiveManagedWorkspace(
    repoPath: string,
    request: ArchiveWorkspaceRequest
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('archive_managed_workspace', { repoPath, request });
  }

  static async restoreManagedWorkspace(repoPath: string, workspaceId: string): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('restore_managed_workspace', { repoPath, workspaceId });
  }

  static async deleteManagedWorkspace(repoPath: string, workspaceId: string): Promise<ApiResponse<boolean>> {
    return await invoke('delete_managed_workspace', { repoPath, workspaceId });
  }

  static async updateManagedWorkspaceStatus(
    repoPath: string,
    workspaceId: string
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('update_managed_workspace_status', { repoPath, workspaceId });
  }

  static async accessManagedWorkspace(repoPath: string, workspaceId: string): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('access_managed_workspace', { repoPath, workspaceId });
  }

  static async addWorkspaceTag(
    repoPath: string,
    workspaceId: string,
    tag: string
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('add_workspace_tag', { repoPath, workspaceId, tag });
  }

  static async removeWorkspaceTag(
    repoPath: string,
    workspaceId: string,
    tag: string
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('remove_workspace_tag', { repoPath, workspaceId, tag });
  }

  static async setWorkspaceCustomField(
    repoPath: string,
    workspaceId: string,
    key: string,
    value: string
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('set_workspace_custom_field', { repoPath, workspaceId, key, value });
  }

  static async removeWorkspaceCustomField(
    repoPath: string,
    workspaceId: string,
    key: string
  ): Promise<ApiResponse<WorkspaceMetadata>> {
    return await invoke('remove_workspace_custom_field', { repoPath, workspaceId, key });
  }

  static async findWorkspacesByTag(repoPath: string, tag: string): Promise<ApiResponse<WorkspaceMetadata[]>> {
    return await invoke('find_workspaces_by_tag', { repoPath, tag });
  }

  static async findWorkspacesByStatus(
    repoPath: string,
    status: WorkspaceStatus
  ): Promise<ApiResponse<WorkspaceMetadata[]>> {
    return await invoke('find_workspaces_by_status', { repoPath, status });
  }

  static async cleanupBrokenWorkspaces(repoPath: string): Promise<ApiResponse<string[]>> {
    return await invoke('cleanup_broken_workspaces', { repoPath });
  }

  static async getWorkspaceStatistics(repoPath: string): Promise<ApiResponse<Record<string, any>>> {
    return await invoke('get_workspace_statistics', { repoPath });
  }

  // 脚本执行
  static async createScriptExecution(
    scriptContent: string,
    workingDirectory: string,
    environment?: Record<string, string>
  ): Promise<ApiResponse<string>> {
    return await invoke('create_script_execution', {
      scriptContent,
      workingDirectory,
      environment: environment || null,
    });
  }

  static async executeScript(executionId: string): Promise<ApiResponse<ScriptExecutionResult>> {
    return await invoke('execute_script', { executionId });
  }

  static async cancelScriptExecution(executionId: string): Promise<ApiResponse<void>> {
    return await invoke('cancel_script_execution', { executionId });
  }

  static async getScriptExecutionStatus(executionId: string): Promise<ApiResponse<ScriptExecution | null>> {
    return await invoke('get_script_execution_status', { executionId });
  }

  static async getAllScriptExecutions(): Promise<ApiResponse<ScriptExecution[]>> {
    return await invoke('get_all_script_executions');
  }

  static async cleanupCompletedScriptExecutions(keepCount: number): Promise<ApiResponse<void>> {
    return await invoke('cleanup_completed_script_executions', { keepCount });
  }

  // 终端服务
  static async createTerminal(
    name?: string,
    workingDirectory: string = '.',
    environment?: Record<string, string>
  ): Promise<ApiResponse<string>> {
    return await invoke('create_terminal', {
      name: name || null,
      workingDirectory,
      environment: environment || null,
    });
  }

  static async startTerminal(terminalId: string): Promise<ApiResponse<void>> {
    return await invoke('start_terminal', { terminalId });
  }

  static async sendTerminalCommand(terminalId: string, command: string): Promise<ApiResponse<void>> {
    return await invoke('send_terminal_command', { terminalId, command });
  }

  static async executeSingleCommand(
    command: string,
    args: string[] = [],
    workingDirectory: string = '.',
    environment?: Record<string, string>
  ): Promise<ApiResponse<TerminalOutput>> {
    return await invoke('execute_single_command', {
      command,
      args,
      workingDirectory,
      environment: environment || null,
    });
  }

  static async getTerminalOutput(terminalId: string): Promise<ApiResponse<TerminalOutput[]>> {
    return await invoke('get_terminal_output', { terminalId });
  }

  static async getTerminalHistory(terminalId: string): Promise<ApiResponse<TerminalOutput[]>> {
    return await invoke('get_terminal_history', { terminalId });
  }

  static async closeTerminal(terminalId: string): Promise<ApiResponse<void>> {
    return await invoke('close_terminal', { terminalId });
  }

  static async getTerminalSession(terminalId: string): Promise<ApiResponse<TerminalSession | null>> {
    return await invoke('get_terminal_session', { terminalId });
  }

  static async getAllTerminals(): Promise<ApiResponse<TerminalSession[]>> {
    return await invoke('get_all_terminals');
  }

  static async cleanupClosedTerminals(): Promise<ApiResponse<void>> {
    return await invoke('cleanup_closed_terminals');
  }

  static async setTerminalName(terminalId: string, name: string): Promise<ApiResponse<void>> {
    return await invoke('set_terminal_name', { terminalId, name });
  }
}