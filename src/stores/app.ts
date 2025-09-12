import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const title = ref('Repository Workspace Manager')
  const isLoading = ref(false)

  function setTitle(newTitle: string) {
    title.value = newTitle
  }

  function setLoading(loading: boolean) {
    isLoading.value = loading
  }

  return {
    title,
    isLoading,
    setTitle,
    setLoading
  }
})