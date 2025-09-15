export interface Repository {
  id: string;
  name: string;
  path: string;
  source_branch: string | null;
  init_script: string | null;
  created_at: string;
  updated_at: string;
}

export interface Workspace {
  id: string;
  repository_id: string;
  name: string;
  branch: string;
  path: string;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
  archived_at: string | null;
}

export interface CreateRepositoryRequest {
  name: string;
  path: string;
  source_branch?: string | null;
  init_script?: string | null;
}

export interface CreateWorkspaceRequest {
  repository_id: string;
  name: string;
  branch: string;
}

export interface UpdateRepositoryRequest {
  name?: string | null;
  source_branch?: string | null;
  init_script?: string | null;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}