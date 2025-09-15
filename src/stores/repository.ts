import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Repository, Workspace } from '@/types/database'
import { DatabaseService } from '@/services/database'

export const useRepositoryStore = defineStore('repository', () => {
  const repositories = ref<Repository[]>([])
  const workspaces = ref<Map<string, Workspace[]>>(new Map())
  const selectedRepository = ref<Repository | null>(null)
  const selectedWorkspace = ref<Workspace | null>(null)
  const isLoading = ref(false)

  async function loadRepositories() {
    isLoading.value = true
    try {
      repositories.value = await DatabaseService.getRepositories()
    } catch (error) {
      console.error('Failed to load repositories:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function createRepository(request: { name: string; path: string; source_branch?: string; init_script?: string }) {
    isLoading.value = true
    try {
      const repository = await DatabaseService.createRepository(request)
      repositories.value.unshift(repository)
      return repository
    } catch (error) {
      console.error('Failed to create repository:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function updateRepository(id: string, request: { name?: string; source_branch?: string; init_script?: string }) {
    isLoading.value = true
    try {
      const updatedRepo = await DatabaseService.updateRepository(id, request)
      if (updatedRepo) {
        const index = repositories.value.findIndex(r => r.id === id)
        if (index !== -1) {
          repositories.value[index] = updatedRepo
        }
        if (selectedRepository.value?.id === id) {
          selectedRepository.value = updatedRepo
        }
      }
      return updatedRepo
    } catch (error) {
      console.error('Failed to update repository:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function deleteRepository(id: string) {
    isLoading.value = true
    try {
      const deleted = await DatabaseService.deleteRepository(id)
      if (deleted) {
        repositories.value = repositories.value.filter(r => r.id !== id)
        workspaces.value.delete(id)
        if (selectedRepository.value?.id === id) {
          selectedRepository.value = null
          selectedWorkspace.value = null
        }
      }
      return deleted
    } catch (error) {
      console.error('Failed to delete repository:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function loadWorkspaces(repositoryId: string) {
    isLoading.value = true
    try {
      const repositoryWorkspaces = await DatabaseService.getWorkspacesByRepository(repositoryId)
      workspaces.value.set(repositoryId, repositoryWorkspaces)
      return repositoryWorkspaces
    } catch (error) {
      console.error('Failed to load workspaces:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function createWorkspace(request: { repository_id: string; name: string; branch: string }, path: string) {
    isLoading.value = true
    try {
      const workspace = await DatabaseService.createWorkspace(request, path)
      const currentWorkspaces = workspaces.value.get(request.repository_id) || []
      workspaces.value.set(request.repository_id, [workspace, ...currentWorkspaces])
      return workspace
    } catch (error) {
      console.error('Failed to create workspace:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function archiveWorkspace(id: string) {
    isLoading.value = true
    try {
      const archivedWorkspace = await DatabaseService.archiveWorkspace(id)
      if (archivedWorkspace) {
        updateWorkspaceInStore(archivedWorkspace)
        if (selectedWorkspace.value?.id === id) {
          selectedWorkspace.value = archivedWorkspace
        }
      }
      return archivedWorkspace
    } catch (error) {
      console.error('Failed to archive workspace:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function restoreWorkspace(id: string) {
    isLoading.value = true
    try {
      const restoredWorkspace = await DatabaseService.restoreWorkspace(id)
      if (restoredWorkspace) {
        updateWorkspaceInStore(restoredWorkspace)
        if (selectedWorkspace.value?.id === id) {
          selectedWorkspace.value = restoredWorkspace
        }
      }
      return restoredWorkspace
    } catch (error) {
      console.error('Failed to restore workspace:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function deleteWorkspace(id: string, repositoryId: string) {
    isLoading.value = true
    try {
      const deleted = await DatabaseService.deleteWorkspace(id)
      if (deleted) {
        const currentWorkspaces = workspaces.value.get(repositoryId) || []
        workspaces.value.set(repositoryId, currentWorkspaces.filter(w => w.id !== id))
        if (selectedWorkspace.value?.id === id) {
          selectedWorkspace.value = null
        }
      }
      return deleted
    } catch (error) {
      console.error('Failed to delete workspace:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  function updateWorkspaceInStore(workspace: Workspace) {
    const currentWorkspaces = workspaces.value.get(workspace.repository_id) || []
    const index = currentWorkspaces.findIndex(w => w.id === workspace.id)
    if (index !== -1) {
      currentWorkspaces[index] = workspace
      workspaces.value.set(workspace.repository_id, [...currentWorkspaces])
    }
  }

  function selectRepository(repository: Repository | null) {
    selectedRepository.value = repository
    selectedWorkspace.value = null
  }

  function selectWorkspace(workspace: Workspace | null) {
    selectedWorkspace.value = workspace
  }

  function getWorkspacesByRepository(repositoryId: string): Workspace[] {
    return workspaces.value.get(repositoryId) || []
  }

  return {
    repositories,
    workspaces,
    selectedRepository,
    selectedWorkspace,
    isLoading,
    loadRepositories,
    createRepository,
    updateRepository,
    deleteRepository,
    loadWorkspaces,
    createWorkspace,
    archiveWorkspace,
    restoreWorkspace,
    deleteWorkspace,
    selectRepository,
    selectWorkspace,
    getWorkspacesByRepository,
  }
})