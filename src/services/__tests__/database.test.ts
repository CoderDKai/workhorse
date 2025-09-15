import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { DatabaseService } from '@/services/database'
import type { ApiResponse, Repository, Workspace } from '@/types/database'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core')

const mockInvoke = vi.mocked(invoke)

describe('DatabaseService', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('healthCheck', () => {
    it('should return true when database is healthy', async () => {
      const mockResponse: ApiResponse<boolean> = {
        success: true,
        data: true,
        error: null
      }
      
      mockInvoke.mockResolvedValue(mockResponse)
      
      const result = await DatabaseService.healthCheck()
      
      expect(result).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('database_health_check')
    })

    it('should throw error when health check fails', async () => {
      const mockResponse: ApiResponse<boolean> = {
        success: false,
        data: null,
        error: 'Database connection failed'
      }
      
      mockInvoke.mockResolvedValue(mockResponse)
      
      await expect(DatabaseService.healthCheck()).rejects.toThrow('Database connection failed')
    })
  })

  describe('createRepository', () => {
    it('should create repository successfully', async () => {
      const mockRepository: Repository = {
        id: 'test-id',
        name: 'Test Repo',
        path: '/test/path',
        source_branch: 'main',
        init_script: 'npm install',
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      const mockResponse: ApiResponse<Repository> = {
        success: true,
        data: mockRepository,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const request = {
        name: 'Test Repo',
        path: '/test/path',
        source_branch: 'main',
        init_script: 'npm install'
      }

      const result = await DatabaseService.createRepository(request)

      expect(result).toEqual(mockRepository)
      expect(mockInvoke).toHaveBeenCalledWith('create_repository', { request })
    })

    it('should throw error when creation fails', async () => {
      const mockResponse: ApiResponse<Repository> = {
        success: false,
        data: null,
        error: 'Repository already exists'
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const request = {
        name: 'Test Repo',
        path: '/test/path'
      }

      await expect(DatabaseService.createRepository(request)).rejects.toThrow('Repository already exists')
    })
  })

  describe('getRepositories', () => {
    it('should return list of repositories', async () => {
      const mockRepositories: Repository[] = [
        {
          id: 'repo-1',
          name: 'Repo 1',
          path: '/path/1',
          source_branch: 'main',
          init_script: null,
          created_at: '2023-01-01T00:00:00Z',
          updated_at: '2023-01-01T00:00:00Z'
        },
        {
          id: 'repo-2',
          name: 'Repo 2',
          path: '/path/2',
          source_branch: 'develop',
          init_script: 'yarn install',
          created_at: '2023-01-02T00:00:00Z',
          updated_at: '2023-01-02T00:00:00Z'
        }
      ]

      const mockResponse: ApiResponse<Repository[]> = {
        success: true,
        data: mockRepositories,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const result = await DatabaseService.getRepositories()

      expect(result).toEqual(mockRepositories)
      expect(result).toHaveLength(2)
      expect(mockInvoke).toHaveBeenCalledWith('get_repositories')
    })

    it('should return empty array when no repositories exist', async () => {
      const mockResponse: ApiResponse<Repository[]> = {
        success: true,
        data: [],
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const result = await DatabaseService.getRepositories()

      expect(result).toEqual([])
      expect(result).toHaveLength(0)
    })
  })

  describe('updateRepository', () => {
    it('should update repository successfully', async () => {
      const mockRepository: Repository = {
        id: 'test-id',
        name: 'Updated Repo',
        path: '/test/path',
        source_branch: 'develop',
        init_script: 'pnpm install',
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T01:00:00Z'
      }

      const mockResponse: ApiResponse<Repository | null> = {
        success: true,
        data: mockRepository,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const request = {
        name: 'Updated Repo',
        source_branch: 'develop',
        init_script: 'pnpm install'
      }

      const result = await DatabaseService.updateRepository('test-id', request)

      expect(result).toEqual(mockRepository)
      expect(mockInvoke).toHaveBeenCalledWith('update_repository', { id: 'test-id', request })
    })

    it('should return null when repository not found', async () => {
      const mockResponse: ApiResponse<Repository | null> = {
        success: true,
        data: null,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const result = await DatabaseService.updateRepository('non-existent', {})

      expect(result).toBeNull()
    })
  })

  describe('workspace operations', () => {
    const mockWorkspace: Workspace = {
      id: 'workspace-id',
      repository_id: 'repo-id',
      name: 'feature-branch',
      branch: 'feature/test',
      path: '/test/workspace',
      is_archived: false,
      created_at: '2023-01-01T00:00:00Z',
      updated_at: '2023-01-01T00:00:00Z',
      archived_at: null
    }

    it('should create workspace successfully', async () => {
      const mockResponse: ApiResponse<Workspace> = {
        success: true,
        data: mockWorkspace,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const request = {
        repository_id: 'repo-id',
        name: 'feature-branch',
        branch: 'feature/test'
      }

      const result = await DatabaseService.createWorkspace(request, '/test/workspace')

      expect(result).toEqual(mockWorkspace)
      expect(mockInvoke).toHaveBeenCalledWith('create_workspace', { request, path: '/test/workspace' })
    })

    it('should get workspaces by repository', async () => {
      const mockWorkspaces: Workspace[] = [mockWorkspace]

      const mockResponse: ApiResponse<Workspace[]> = {
        success: true,
        data: mockWorkspaces,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const result = await DatabaseService.getWorkspacesByRepository('repo-id')

      expect(result).toEqual(mockWorkspaces)
      expect(mockInvoke).toHaveBeenCalledWith('get_workspaces_by_repository', { repository_id: 'repo-id' })
    })

    it('should archive workspace', async () => {
      const archivedWorkspace = {
        ...mockWorkspace,
        is_archived: true,
        archived_at: '2023-01-01T01:00:00Z'
      }

      const mockResponse: ApiResponse<Workspace | null> = {
        success: true,
        data: archivedWorkspace,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const result = await DatabaseService.archiveWorkspace('workspace-id')

      expect(result).toEqual(archivedWorkspace)
      expect(result?.is_archived).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('archive_workspace', { id: 'workspace-id' })
    })

    it('should restore workspace', async () => {
      const restoredWorkspace = {
        ...mockWorkspace,
        is_archived: false,
        archived_at: null
      }

      const mockResponse: ApiResponse<Workspace | null> = {
        success: true,
        data: restoredWorkspace,
        error: null
      }

      mockInvoke.mockResolvedValue(mockResponse)

      const result = await DatabaseService.restoreWorkspace('workspace-id')

      expect(result).toEqual(restoredWorkspace)
      expect(result?.is_archived).toBe(false)
      expect(mockInvoke).toHaveBeenCalledWith('restore_workspace', { id: 'workspace-id' })
    })
  })

  describe('error handling', () => {
    it('should handle Tauri invoke errors', async () => {
      mockInvoke.mockRejectedValue(new Error('Tauri invoke failed'))

      await expect(DatabaseService.healthCheck()).rejects.toThrow('Tauri invoke failed')
    })

    it('should handle malformed responses', async () => {
      mockInvoke.mockResolvedValue({
        success: false,
        data: null,
        error: null // Error message is null but success is false
      })

      await expect(DatabaseService.healthCheck()).rejects.toThrow('Database health check failed')
    })
  })
})