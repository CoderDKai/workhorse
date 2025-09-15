import { describe, it, expect } from 'vitest'
import type { 
  Repository, 
  Workspace, 
  CreateRepositoryRequest, 
  CreateWorkspaceRequest, 
  UpdateRepositoryRequest,
  ApiResponse 
} from '@/types/database'

describe('Database Types', () => {
  describe('Repository', () => {
    it('should have correct structure', () => {
      const repository: Repository = {
        id: 'repo-id',
        name: 'Test Repository',
        path: '/path/to/repo',
        source_branch: 'main',
        init_script: 'npm install',
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      expect(repository).toHaveProperty('id')
      expect(repository).toHaveProperty('name')
      expect(repository).toHaveProperty('path')
      expect(repository).toHaveProperty('source_branch')
      expect(repository).toHaveProperty('init_script')
      expect(repository).toHaveProperty('created_at')
      expect(repository).toHaveProperty('updated_at')

      expect(typeof repository.id).toBe('string')
      expect(typeof repository.name).toBe('string')
      expect(typeof repository.path).toBe('string')
      expect(typeof repository.created_at).toBe('string')
      expect(typeof repository.updated_at).toBe('string')
    })

    it('should allow null values for optional fields', () => {
      const repository: Repository = {
        id: 'repo-id',
        name: 'Test Repository',
        path: '/path/to/repo',
        source_branch: null,
        init_script: null,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      expect(repository.source_branch).toBeNull()
      expect(repository.init_script).toBeNull()
    })
  })

  describe('Workspace', () => {
    it('should have correct structure', () => {
      const workspace: Workspace = {
        id: 'workspace-id',
        repository_id: 'repo-id',
        name: 'feature-branch',
        branch: 'feature/test',
        path: '/path/to/workspace',
        is_archived: false,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z',
        archived_at: null
      }

      expect(workspace).toHaveProperty('id')
      expect(workspace).toHaveProperty('repository_id')
      expect(workspace).toHaveProperty('name')
      expect(workspace).toHaveProperty('branch')
      expect(workspace).toHaveProperty('path')
      expect(workspace).toHaveProperty('is_archived')
      expect(workspace).toHaveProperty('created_at')
      expect(workspace).toHaveProperty('updated_at')
      expect(workspace).toHaveProperty('archived_at')

      expect(typeof workspace.id).toBe('string')
      expect(typeof workspace.repository_id).toBe('string')
      expect(typeof workspace.name).toBe('string')
      expect(typeof workspace.branch).toBe('string')
      expect(typeof workspace.path).toBe('string')
      expect(typeof workspace.is_archived).toBe('boolean')
      expect(typeof workspace.created_at).toBe('string')
      expect(typeof workspace.updated_at).toBe('string')
    })

    it('should support archived state', () => {
      const archivedWorkspace: Workspace = {
        id: 'workspace-id',
        repository_id: 'repo-id',
        name: 'archived-branch',
        branch: 'feature/archived',
        path: '/path/to/workspace',
        is_archived: true,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T01:00:00Z',
        archived_at: '2023-01-01T01:00:00Z'
      }

      expect(archivedWorkspace.is_archived).toBe(true)
      expect(archivedWorkspace.archived_at).toBe('2023-01-01T01:00:00Z')
    })
  })

  describe('Request Types', () => {
    it('CreateRepositoryRequest should have correct structure', () => {
      const request: CreateRepositoryRequest = {
        name: 'New Repository',
        path: '/path/to/repo',
        source_branch: 'main',
        init_script: 'npm install'
      }

      expect(request).toHaveProperty('name')
      expect(request).toHaveProperty('path')
      expect(request).toHaveProperty('source_branch')
      expect(request).toHaveProperty('init_script')
    })

    it('CreateRepositoryRequest should allow optional fields', () => {
      const minimalRequest: CreateRepositoryRequest = {
        name: 'New Repository',
        path: '/path/to/repo'
      }

      expect(minimalRequest.name).toBe('New Repository')
      expect(minimalRequest.path).toBe('/path/to/repo')
      expect(minimalRequest.source_branch).toBeUndefined()
      expect(minimalRequest.init_script).toBeUndefined()
    })

    it('CreateWorkspaceRequest should have correct structure', () => {
      const request: CreateWorkspaceRequest = {
        repository_id: 'repo-id',
        name: 'feature-workspace',
        branch: 'feature/new'
      }

      expect(request).toHaveProperty('repository_id')
      expect(request).toHaveProperty('name')
      expect(request).toHaveProperty('branch')
      expect(typeof request.repository_id).toBe('string')
      expect(typeof request.name).toBe('string')
      expect(typeof request.branch).toBe('string')
    })

    it('UpdateRepositoryRequest should allow partial updates', () => {
      const updateName: UpdateRepositoryRequest = {
        name: 'Updated Name'
      }

      const updateBranch: UpdateRepositoryRequest = {
        source_branch: 'develop'
      }

      const updateScript: UpdateRepositoryRequest = {
        init_script: 'pnpm install'
      }

      const updateAll: UpdateRepositoryRequest = {
        name: 'Updated Repository',
        source_branch: 'main',
        init_script: 'yarn install'
      }

      expect(updateName.name).toBe('Updated Name')
      expect(updateName.source_branch).toBeUndefined()
      expect(updateName.init_script).toBeUndefined()

      expect(updateBranch.source_branch).toBe('develop')
      expect(updateScript.init_script).toBe('pnpm install')
      
      expect(updateAll.name).toBe('Updated Repository')
      expect(updateAll.source_branch).toBe('main')
      expect(updateAll.init_script).toBe('yarn install')
    })
  })

  describe('ApiResponse', () => {
    it('should handle successful response', () => {
      const successResponse: ApiResponse<string> = {
        success: true,
        data: 'test data',
        error: null
      }

      expect(successResponse.success).toBe(true)
      expect(successResponse.data).toBe('test data')
      expect(successResponse.error).toBeNull()
    })

    it('should handle error response', () => {
      const errorResponse: ApiResponse<string> = {
        success: false,
        data: null,
        error: 'Something went wrong'
      }

      expect(errorResponse.success).toBe(false)
      expect(errorResponse.data).toBeNull()
      expect(errorResponse.error).toBe('Something went wrong')
    })

    it('should work with different data types', () => {
      const repositoryResponse: ApiResponse<Repository> = {
        success: true,
        data: {
          id: 'repo-id',
          name: 'Test Repo',
          path: '/path',
          source_branch: 'main',
          init_script: null,
          created_at: '2023-01-01T00:00:00Z',
          updated_at: '2023-01-01T00:00:00Z'
        },
        error: null
      }

      const repositoryListResponse: ApiResponse<Repository[]> = {
        success: true,
        data: [repositoryResponse.data!],
        error: null
      }

      const booleanResponse: ApiResponse<boolean> = {
        success: true,
        data: true,
        error: null
      }

      expect(repositoryResponse.data).toHaveProperty('id')
      expect(repositoryListResponse.data).toHaveLength(1)
      expect(booleanResponse.data).toBe(true)
    })
  })
})