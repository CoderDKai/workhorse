import { invoke } from '@tauri-apps/api/core';
import type {
  Repository,
  Workspace,
  CreateRepositoryRequest,
  CreateWorkspaceRequest,
  UpdateRepositoryRequest,
  ApiResponse,
} from '@/types/database';

export class DatabaseService {
  static async healthCheck(): Promise<boolean> {
    const response = await invoke<ApiResponse<boolean>>('database_health_check');
    if (!response.success) {
      throw new Error(response.error || 'Database health check failed');
    }
    return response.data || false;
  }

  static async createRepository(request: CreateRepositoryRequest): Promise<Repository> {
    const response = await invoke<ApiResponse<Repository>>('create_repository', { request });
    if (!response.success) {
      throw new Error(response.error || 'Failed to create repository');
    }
    return response.data!;
  }

  static async getRepositories(): Promise<Repository[]> {
    const response = await invoke<ApiResponse<Repository[]>>('get_repositories');
    if (!response.success) {
      throw new Error(response.error || 'Failed to get repositories');
    }
    return response.data || [];
  }

  static async getRepositoryById(id: string): Promise<Repository | null> {
    const response = await invoke<ApiResponse<Repository | null>>('get_repository_by_id', { id });
    if (!response.success) {
      throw new Error(response.error || 'Failed to get repository');
    }
    return response.data;
  }

  static async updateRepository(id: string, request: UpdateRepositoryRequest): Promise<Repository | null> {
    const response = await invoke<ApiResponse<Repository | null>>('update_repository', { id, request });
    if (!response.success) {
      throw new Error(response.error || 'Failed to update repository');
    }
    return response.data;
  }

  static async deleteRepository(id: string): Promise<boolean> {
    const response = await invoke<ApiResponse<boolean>>('delete_repository', { id });
    if (!response.success) {
      throw new Error(response.error || 'Failed to delete repository');
    }
    return response.data || false;
  }

  static async createWorkspace(request: CreateWorkspaceRequest, path: string): Promise<Workspace> {
    const response = await invoke<ApiResponse<Workspace>>('create_workspace', { request, path });
    if (!response.success) {
      throw new Error(response.error || 'Failed to create workspace');
    }
    return response.data!;
  }

  static async getWorkspacesByRepository(repositoryId: string): Promise<Workspace[]> {
    const response = await invoke<ApiResponse<Workspace[]>>('get_workspaces_by_repository', { 
      repository_id: repositoryId 
    });
    if (!response.success) {
      throw new Error(response.error || 'Failed to get workspaces');
    }
    return response.data || [];
  }

  static async archiveWorkspace(id: string): Promise<Workspace | null> {
    const response = await invoke<ApiResponse<Workspace | null>>('archive_workspace', { id });
    if (!response.success) {
      throw new Error(response.error || 'Failed to archive workspace');
    }
    return response.data;
  }

  static async restoreWorkspace(id: string): Promise<Workspace | null> {
    const response = await invoke<ApiResponse<Workspace | null>>('restore_workspace', { id });
    if (!response.success) {
      throw new Error(response.error || 'Failed to restore workspace');
    }
    return response.data;
  }

  static async deleteWorkspace(id: string): Promise<boolean> {
    const response = await invoke<ApiResponse<boolean>>('delete_workspace', { id });
    if (!response.success) {
      throw new Error(response.error || 'Failed to delete workspace');
    }
    return response.data || false;
  }
}