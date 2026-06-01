import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project } from '@/types'
import { useWorkspaceStore } from './workspace'
import * as api from '@/api'

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)
  const loading = ref(false)

  const workspaceProjects = computed(() => {
    const workspaceStore = useWorkspaceStore()
    if (!workspaceStore.currentWorkspace) return []
    return projects.value.filter(
      (p) => p.workspaceId === workspaceStore.currentWorkspace!.id
    )
  })

  async function fetchProjects() {
    loading.value = true
    try {
      projects.value = await api.listProjects()
    } finally {
      loading.value = false
    }
  }

  async function createProject(name: string, path: string, description?: string) {
    const workspaceStore = useWorkspaceStore()
    if (!workspaceStore.currentWorkspace) {
      throw new Error('请先初始化工作区')
    }
    const now = new Date().toISOString()
    const project = await api.createProject(
      crypto.randomUUID(),
      workspaceStore.currentWorkspace.id,
      name,
      path,
      description,
      now,
      now
    )
    projects.value.push(project)
    return project
  }

  async function deleteProject(id: string) {
    await api.deleteProject(id)
    projects.value = projects.value.filter((p) => p.id !== id)
    if (currentProject.value?.id === id) {
      currentProject.value = null
    }
  }

  function setCurrentProject(project: Project | null) {
    currentProject.value = project
  }

  return {
    projects,
    currentProject,
    loading,
    workspaceProjects,
    fetchProjects,
    createProject,
    deleteProject,
    setCurrentProject,
  }
})
