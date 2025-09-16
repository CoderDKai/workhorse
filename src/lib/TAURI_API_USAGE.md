# Tauri API 使用指南

本文档描述如何使用 TauriAPIClient 来调用后端服务。

## 导入

```typescript
import { TauriAPIClient } from '@/lib/tauri-api';
import type { Repository, GitStatus, WorkspaceMetadata } from '@/types';
```

## 基础用法

### 数据库健康检查

```typescript
const healthResponse = await TauriAPIClient.databaseHealthCheck();
if (healthResponse.success) {
  console.log('数据库状态:', healthResponse.data);
} else {
  console.error('健康检查失败:', healthResponse.error);
}
```

### 仓库管理

```typescript
// 创建仓库
const createRepoResponse = await TauriAPIClient.createRepository({
  name: 'My Project',
  path: '/path/to/project',
  source_branch: 'main',
  init_script: 'npm install'
});

// 获取所有仓库
const reposResponse = await TauriAPIClient.getRepositories();
if (reposResponse.success && reposResponse.data) {
  const repositories = reposResponse.data;
  console.log('所有仓库:', repositories);
}
```

### Git 操作

```typescript
// 检查 Git 状态
const gitStatusResponse = await TauriAPIClient.getGitStatus('/path/to/repo');
if (gitStatusResponse.success && gitStatusResponse.data) {
  const status = gitStatusResponse.data;
  console.log('是否干净:', status.is_clean);
  console.log('修改的文件:', status.modified_files);
}

// 获取分支列表
const branchesResponse = await TauriAPIClient.getGitBranches('/path/to/repo');
if (branchesResponse.success && branchesResponse.data) {
  const branches = branchesResponse.data;
  const currentBranch = branches.find(b => b.is_head);
  console.log('当前分支:', currentBranch?.name);
}

// 创建工作树
const worktreeResponse = await TauriAPIClient.createGitWorktree(
  '/path/to/repo',
  'feature-branch',
  '/path/to/worktree',
  'feature/new-feature'
);
```

### 仓库管理服务

```typescript
// 验证仓库
const validationResponse = await TauriAPIClient.validateRepository('/path/to/repo');
if (validationResponse.success && validationResponse.data) {
  const validation = validationResponse.data;
  if (!validation.is_valid) {
    console.error('仓库验证失败:', validation.errors);
  }
}

// 添加仓库到管理
const addRepoResponse = await TauriAPIClient.addRepositoryManagement({
  path: '/path/to/repo',
  default_branch: 'main',
  setup_workhorse_dir: true
});
```

### 工作区管理

```typescript
// 创建工作区
const createWorkspaceResponse = await TauriAPIClient.createManagedWorkspace(
  '/path/to/repo',
  {
    name: 'Feature Work',
    description: '新功能开发',
    branch_name: 'feature/new-feature',
    create_branch_if_not_exists: true,
    tags: ['development', 'feature'],
    custom_fields: { priority: 'high' }
  }
);

// 列出工作区
const workspacesResponse = await TauriAPIClient.listManagedWorkspaces('/path/to/repo');
if (workspacesResponse.success && workspacesResponse.data) {
  const workspaces = workspacesResponse.data;
  console.log('工作区数量:', workspaces.length);
}

// 获取工作区详细信息
const workspaceInfoResponse = await TauriAPIClient.getManagedWorkspaceInfo(
  '/path/to/repo',
  'workspace-id'
);
```

### 脚本执行

```typescript
// 创建脚本执行
const executionId = await TauriAPIClient.createScriptExecution(
  'npm install && npm run build',
  '/path/to/working/directory',
  { NODE_ENV: 'production' }
);

if (executionId.success && executionId.data) {
  // 执行脚本
  const result = await TauriAPIClient.executeScript(executionId.data);
  if (result.success && result.data) {
    console.log('执行结果:', result.data.stdout);
    console.log('退出码:', result.data.exit_code);
  }
}
```

### 终端操作

```typescript
// 创建终端
const terminalResponse = await TauriAPIClient.createTerminal(
  'Build Terminal',
  '/path/to/project',
  { NODE_ENV: 'development' }
);

if (terminalResponse.success && terminalResponse.data) {
  const terminalId = terminalResponse.data;
  
  // 启动终端
  await TauriAPIClient.startTerminal(terminalId);
  
  // 发送命令
  await TauriAPIClient.sendTerminalCommand(terminalId, 'npm run dev');
  
  // 获取输出
  const outputResponse = await TauriAPIClient.getTerminalOutput(terminalId);
  if (outputResponse.success && outputResponse.data) {
    const outputs = outputResponse.data;
    outputs.forEach(output => {
      console.log(`[${output.timestamp}] ${output.content}`);
    });
  }
}
```

## 错误处理

所有 API 调用都返回 `ApiResponse<T>` 类型，包含 `success`、`data` 和 `error` 字段：

```typescript
const response = await TauriAPIClient.someApiCall();

if (response.success) {
  // 成功情况
  const data = response.data; // T | null
  if (data) {
    // 处理数据
  }
} else {
  // 错误情况
  console.error('API 调用失败:', response.error);
}
```

## 类型安全

所有 API 方法都提供完整的 TypeScript 类型支持：

```typescript
// 类型会被自动推断
const repos: ApiResponse<Repository[]> = await TauriAPIClient.getRepositories();

// 类型检查会确保正确的参数类型
const workspace: ApiResponse<WorkspaceMetadata> = await TauriAPIClient.createManagedWorkspace(
  '/valid/path',
  {
    name: 'Required String',
    description: null, // 可选
    branch_name: 'Required String',
    create_branch_if_not_exists: true, // boolean
    tags: ['array', 'of', 'strings'],
    custom_fields: {} // Record<string, string>
  }
);
```