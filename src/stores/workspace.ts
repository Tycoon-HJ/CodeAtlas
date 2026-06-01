import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Workspace } from '@/types'
import * as api from '@/api'

const DEFAULT_WORKSPACE_ID = 'default'

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentWorkspace = ref<Workspace | null>(null)
  const loading = ref(false)
  const initialized = ref(false)

  async function init() {
    if (initialized.value) return
    loading.value = true
    try {
      const existing = await api.listWorkspaces()
      if (existing.length > 0) {
        workspaces.value = existing
        currentWorkspace.value = existing[0]
      } else {
        // Auto-create default workspace
        const now = new Date().toISOString()
        const ws = await api.createWorkspace(
          DEFAULT_WORKSPACE_ID,
          '默认工作区',
          '个人项目工作区',
          now,
          now
        )
        workspaces.value = [ws]
        currentWorkspace.value = ws
      }
      initialized.value = true
    } catch (e) {
      console.error('Failed to init workspace:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchWorkspaces() {
    loading.value = true
    try {
      workspaces.value = await api.listWorkspaces()
      if (workspaces.value.length > 0 && !currentWorkspace.value) {
        currentWorkspace.value = workspaces.value[0]
      }
    } finally {
      loading.value = false
    }
  }

  async function createWorkspace(name: string, description?: string) {
    const now = new Date().toISOString()
    const ws = await api.createWorkspace(crypto.randomUUID(), name, description, now, now)
    workspaces.value.push(ws)
    if (!currentWorkspace.value) {
      currentWorkspace.value = ws
    }
    return ws
  }

  async function deleteWorkspace(id: string) {
    await api.deleteWorkspace(id)
    workspaces.value = workspaces.value.filter((w) => w.id !== id)
    if (currentWorkspace.value?.id === id) {
      currentWorkspace.value = workspaces.value[0] ?? null
    }
  }

  function setCurrentWorkspace(workspace: Workspace | null) {
    currentWorkspace.value = workspace
  }

  return {
    workspaces,
    currentWorkspace,
    loading,
    initialized,
    init,
    fetchWorkspaces,
    createWorkspace,
    deleteWorkspace,
    setCurrentWorkspace,
  }
})
