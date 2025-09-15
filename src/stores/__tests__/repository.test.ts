import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useRepositoryStore } from '@/stores/repository'
import { DatabaseService } from '@/services/database'
import type { Repository, Workspace } from '@/types/database'

// Mock DatabaseService
vi.mock('@/services/database')
const mockDatabaseService = vi.mocked(DatabaseService)

describe('useRepositoryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have correct initial state', () => {
      const store = useRepositoryStore()
      
      expect(store.repositories).toEqual([])
      expect(store.workspaces).toBeInstanceOf(Map)
      expect(store.workspaces.size).toBe(0)
      expect(store.selectedRepository).toBeNull()
      expect(store.selectedWorkspace).toBeNull()
      expect(store.isLoading).toBe(false)
    })
  })

  describe('loadRepositories', () => {
    it('should load repositories successfully', async () => {
      const mockRepositories: Repository[] = [
        {
          id: 'repo-1',
          name: 'Test Repo 1',
          path: '/path/1',
          source_branch: 'main',
          init_script: null,
          created_at: '2023-01-01T00:00:00Z',
          updated_at: '2023-01-01T00:00:00Z'
        },
        {
          id: 'repo-2',
          name: 'Test Repo 2',
          path: '/path/2',
          source_branch: 'develop',
          init_script: 'npm install',
          created_at: '2023-01-02T00:00:00Z',
          updated_at: '2023-01-02T00:00:00Z'
        }
      ]

      mockDatabaseService.getRepositories.mockResolvedValue(mockRepositories)

      const store = useRepositoryStore()
      await store.loadRepositories()

      expect(store.repositories).toEqual(mockRepositories)
      expect(store.isLoading).toBe(false)
      expect(mockDatabaseService.getRepositories).toHaveBeenCalled()
    })

    it('should handle loading state correctly', async () => {
      mockDatabaseService.getRepositories.mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve([]), 100))
      )

      const store = useRepositoryStore()
      const loadPromise = store.loadRepositories()

      expect(store.isLoading).toBe(true)

      await loadPromise

      expect(store.isLoading).toBe(false)
    })

    it('should handle errors and still reset loading state', async () => {
      mockDatabaseService.getRepositories.mockRejectedValue(new Error('Database error'))

      const store = useRepositoryStore()

      await expect(store.loadRepositories()).rejects.toThrow('Database error')
      expect(store.isLoading).toBe(false)
    })
  })

  describe('createRepository', () => {
    it('should create repository and add to store', async () => {
      const newRepo: Repository = {
        id: 'new-repo',
        name: 'New Repo',
        path: '/new/path',
        source_branch: 'main',
        init_script: 'pnpm install',
        created_at: '2023-01-03T00:00:00Z',
        updated_at: '2023-01-03T00:00:00Z'
      }

      mockDatabaseService.createRepository.mockResolvedValue(newRepo)

      const store = useRepositoryStore()
      const request = {
        name: 'New Repo',
        path: '/new/path',
        source_branch: 'main',
        init_script: 'pnpm install'
      }

      const result = await store.createRepository(request)

      expect(result).toEqual(newRepo)
      expect(store.repositories).toHaveLength(1)
      expect(store.repositories[0]).toEqual(newRepo)
      expect(mockDatabaseService.createRepository).toHaveBeenCalledWith(request)
    })
  })

  describe('updateRepository', () => {
    it('should update repository in store', async () => {
      const originalRepo: Repository = {
        id: 'repo-1',
        name: 'Original Name',
        path: '/path/1',
        source_branch: 'main',
        init_script: null,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      const updatedRepo: Repository = {
        ...originalRepo,
        name: 'Updated Name',
        updated_at: '2023-01-01T01:00:00Z'
      }

      mockDatabaseService.updateRepository.mockResolvedValue(updatedRepo)

      const store = useRepositoryStore()
      store.repositories = [originalRepo]

      const result = await store.updateRepository('repo-1', { name: 'Updated Name' })

      expect(result).toEqual(updatedRepo)
      expect(store.repositories[0]).toEqual(updatedRepo)
    })

    it('should update selectedRepository if it matches', async () => {
      const originalRepo: Repository = {
        id: 'repo-1',
        name: 'Original Name',
        path: '/path/1',
        source_branch: 'main',
        init_script: null,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      const updatedRepo: Repository = {
        ...originalRepo,
        name: 'Updated Name'
      }

      mockDatabaseService.updateRepository.mockResolvedValue(updatedRepo)

      const store = useRepositoryStore()
      store.repositories = [originalRepo]
      store.selectedRepository = originalRepo

      await store.updateRepository('repo-1', { name: 'Updated Name' })

      expect(store.selectedRepository).toEqual(updatedRepo)
    })
  })

  describe('deleteRepository', () => {
    it('should delete repository from store', async () => {
      const repo1: Repository = {
        id: 'repo-1',
        name: 'Repo 1',
        path: '/path/1',
        source_branch: 'main',
        init_script: null,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      const repo2: Repository = {
        id: 'repo-2',
        name: 'Repo 2',
        path: '/path/2',
        source_branch: 'develop',
        init_script: null,
        created_at: '2023-01-02T00:00:00Z',
        updated_at: '2023-01-02T00:00:00Z'
      }

      mockDatabaseService.deleteRepository.mockResolvedValue(true)

      const store = useRepositoryStore()
      store.repositories = [repo1, repo2]
      store.workspaces.set('repo-1', [])

      const result = await store.deleteRepository('repo-1')

      expect(result).toBe(true)
      expect(store.repositories).toHaveLength(1)
      expect(store.repositories[0]).toEqual(repo2)
      expect(store.workspaces.has('repo-1')).toBe(false)
    })

    it('should clear selections if deleted repository was selected', async () => {
      const repo: Repository = {
        id: 'repo-1',
        name: 'Repo 1',
        path: '/path/1',
        source_branch: 'main',
        init_script: null,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      mockDatabaseService.deleteRepository.mockResolvedValue(true)

      const store = useRepositoryStore()
      store.repositories = [repo]
      store.selectedRepository = repo
      store.selectedWorkspace = {
        id: 'ws-1',
        repository_id: 'repo-1',
        name: 'workspace',
        branch: 'feature',
        path: '/ws/path',
        is_archived: false,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z',
        archived_at: null
      }

      await store.deleteRepository('repo-1')

      expect(store.selectedRepository).toBeNull()
      expect(store.selectedWorkspace).toBeNull()
    })
  })

  describe('workspace operations', () => {
    const mockWorkspace: Workspace = {
      id: 'ws-1',
      repository_id: 'repo-1',
      name: 'feature-branch',
      branch: 'feature/test',
      path: '/workspace/path',
      is_archived: false,
      created_at: '2023-01-01T00:00:00Z',
      updated_at: '2023-01-01T00:00:00Z',
      archived_at: null
    }

    it('should load workspaces for repository', async () => {
      const workspaces = [mockWorkspace]
      mockDatabaseService.getWorkspacesByRepository.mockResolvedValue(workspaces)

      const store = useRepositoryStore()
      const result = await store.loadWorkspaces('repo-1')

      expect(result).toEqual(workspaces)
      expect(store.workspaces.get('repo-1')).toEqual(workspaces)
      expect(mockDatabaseService.getWorkspacesByRepository).toHaveBeenCalledWith('repo-1')
    })

    it('should create workspace and add to store', async () => {
      mockDatabaseService.createWorkspace.mockResolvedValue(mockWorkspace)

      const store = useRepositoryStore()
      const request = {
        repository_id: 'repo-1',
        name: 'feature-branch',
        branch: 'feature/test'
      }

      const result = await store.createWorkspace(request, '/workspace/path')

      expect(result).toEqual(mockWorkspace)
      expect(store.workspaces.get('repo-1')).toEqual([mockWorkspace])
    })

    it('should archive workspace', async () => {
      const archivedWorkspace = { ...mockWorkspace, is_archived: true, archived_at: '2023-01-01T01:00:00Z' }
      mockDatabaseService.archiveWorkspace.mockResolvedValue(archivedWorkspace)

      const store = useRepositoryStore()
      store.workspaces.set('repo-1', [mockWorkspace])

      const result = await store.archiveWorkspace('ws-1')

      expect(result).toEqual(archivedWorkspace)
      
      const updatedWorkspaces = store.workspaces.get('repo-1')
      expect(updatedWorkspaces?.[0].is_archived).toBe(true)
    })

    it('should restore workspace', async () => {
      const archivedWorkspace = { ...mockWorkspace, is_archived: true, archived_at: '2023-01-01T01:00:00Z' }
      const restoredWorkspace = { ...mockWorkspace, is_archived: false, archived_at: null }
      
      mockDatabaseService.restoreWorkspace.mockResolvedValue(restoredWorkspace)

      const store = useRepositoryStore()
      store.workspaces.set('repo-1', [archivedWorkspace])

      const result = await store.restoreWorkspace('ws-1')

      expect(result).toEqual(restoredWorkspace)
      
      const updatedWorkspaces = store.workspaces.get('repo-1')
      expect(updatedWorkspaces?.[0].is_archived).toBe(false)
    })

    it('should delete workspace from store', async () => {
      const workspace2 = { ...mockWorkspace, id: 'ws-2', name: 'other-workspace' }
      mockDatabaseService.deleteWorkspace.mockResolvedValue(true)

      const store = useRepositoryStore()
      store.workspaces.set('repo-1', [mockWorkspace, workspace2])

      const result = await store.deleteWorkspace('ws-1', 'repo-1')

      expect(result).toBe(true)
      expect(store.workspaces.get('repo-1')).toEqual([workspace2])
    })
  })

  describe('selection management', () => {
    it('should select repository', () => {
      const repo: Repository = {
        id: 'repo-1',
        name: 'Test Repo',
        path: '/path',
        source_branch: 'main',
        init_script: null,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z'
      }

      const store = useRepositoryStore()
      store.selectRepository(repo)

      expect(store.selectedRepository).toEqual(repo)
      expect(store.selectedWorkspace).toBeNull()
    })

    it('should select workspace', () => {
      const workspace: Workspace = {
        id: 'ws-1',
        repository_id: 'repo-1',
        name: 'workspace',
        branch: 'feature',
        path: '/path',
        is_archived: false,
        created_at: '2023-01-01T00:00:00Z',
        updated_at: '2023-01-01T00:00:00Z',
        archived_at: null
      }

      const store = useRepositoryStore()
      store.selectWorkspace(workspace)

      expect(store.selectedWorkspace).toEqual(workspace)
    })

    it('should get workspaces by repository', () => {
      const workspaces = [
        {
          id: 'ws-1',
          repository_id: 'repo-1',
          name: 'workspace-1',
          branch: 'feature-1',
          path: '/path/1',
          is_archived: false,
          created_at: '2023-01-01T00:00:00Z',
          updated_at: '2023-01-01T00:00:00Z',
          archived_at: null
        }
      ]

      const store = useRepositoryStore()
      store.workspaces.set('repo-1', workspaces)

      const result = store.getWorkspacesByRepository('repo-1')
      expect(result).toEqual(workspaces)

      const emptyResult = store.getWorkspacesByRepository('repo-2')
      expect(emptyResult).toEqual([])
    })
  })
})