<script setup lang="ts">
import { NCard, NGrid, NGi, NText, NButton, NSpace, NInput, NEmpty, NTag, NPopconfirm } from 'naive-ui'
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Project } from '@/types'

const router = useRouter()
const projectStore = useProjectStore()
const workspaceStore = useWorkspaceStore()
const searchQuery = ref('')

onMounted(async () => {
  await workspaceStore.init()
  await projectStore.fetchProjects()
})

const filteredProjects = computed(() => {
  const q = searchQuery.value.toLowerCase()
  let list = projectStore.projects
  if (q) {
    list = list.filter(
      (p) => p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q)
    )
  }
  return list.sort((a, b) => {
    const ta = a.lastOpenTime ?? a.updatedAt
    const tb = b.lastOpenTime ?? b.updatedAt
    return tb.localeCompare(ta)
  })
})

const recentProjects = computed(() => filteredProjects.value.slice(0, 8))

function openProject(project: Project) {
  projectStore.setCurrentProject(project)
  router.push(`/project/${project.id}`)
}

function formatTime(dateStr: string): string {
  if (!dateStr) return ''
  // Handle both ISO string and millisecond timestamp
  const d = /^\d+$/.test(dateStr) ? new Date(Number(dateStr)) : new Date(dateStr)
  if (isNaN(d.getTime())) return ''
  const year = d.getFullYear()
  const month = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const hours = String(d.getHours()).padStart(2, '0')
  const minutes = String(d.getMinutes()).padStart(2, '0')
  const seconds = String(d.getSeconds()).padStart(2, '0')
  return `${year}/${month}/${day} ${hours}:${minutes}:${seconds}`
}

async function handleOpenDirectory() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      // 检查是否已有相同路径的项目
      const existing = projectStore.projects.find((p) => p.path === selected)
      if (existing) {
        openProject(existing)
        return
      }
      const name = selected.split(/[\\/]/).pop() ?? selected
      const project = await projectStore.createProject(name, selected)
      openProject(project)
    }
  } catch (e) {
    console.error('Failed to open directory:', e)
  }
}

async function handleDeleteProject(id: string) {
  await projectStore.deleteProject(id)
}
</script>

<template>
  <div class="welcome-view">
    <div class="welcome-header">
      <div class="welcome-logo">
        <div class="logo-mark">CA</div>
        <div class="logo-text">
          <h1 class="welcome-title">CodeAtlas Studio</h1>
          <p class="welcome-subtitle">AI Coding Workspace — 打开目录开始工作</p>
        </div>
      </div>
    </div>

    <div class="actions-bar">
      <button class="open-btn" @click="handleOpenDirectory">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h4l1-1h6a1 1 0 011 1v8a1 1 0 01-1 1H2a1 1 0 01-1-1V5a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
        </svg>
        打开目录
      </button>
      <div class="search-wrapper">
        <svg class="search-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
          <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.2"/>
          <path d="M11 11l3 3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
        <input
          v-model="searchQuery"
          class="search-input"
          placeholder="搜索项目..."
        />
      </div>
    </div>

    <!-- Recent projects -->
    <div v-if="recentProjects.length > 0" class="section">
      <div class="section-label">最近项目</div>
      <div class="projects-grid">
        <div
          v-for="project in recentProjects"
          :key="project.id"
          class="project-card"
          @click="openProject(project)"
        >
          <div class="project-card-main">
            <div class="project-icon">
              <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
                <path d="M2 4h4l1-1h6a1 1 0 011 1v8a1 1 0 01-1 1H2a1 1 0 01-1-1V5a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
              </svg>
            </div>
            <div class="project-info">
              <div class="project-name-row">
                <span class="project-name">{{ project.name }}</span>
                <NTag v-if="project.isFavorite" size="small" :bordered="false" style="background: var(--warning-light); color: var(--warning); font-size: 10px">收藏</NTag>
              </div>
              <div class="project-path">{{ project.path }}</div>
            </div>
            <NPopconfirm @positive-click="handleDeleteProject(project.id)">
              <template #trigger>
                <button class="project-delete" @click.stop title="删除项目">
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                    <path d="M3 4h10M6 4V3a1 1 0 011-1h2a1 1 0 011 1v1M5 4v8a1 1 0 001 1h4a1 1 0 001-1V4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
              </template>
              确定删除项目「{{ project.name }}」？此操作不可撤销。
            </NPopconfirm>
          </div>
          <div class="project-meta">
            <span v-if="project.branch" class="project-branch">{{ project.branch }}</span>
            <span class="project-time">{{ formatTime(project.lastOpenTime ?? project.updatedAt) }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <svg width="48" height="48" viewBox="0 0 48 48" fill="none" style="opacity: 0.2">
        <path d="M6 12h12l3-3h18a3 3 0 013 3v24a3 3 0 01-3 3H6a3 3 0 01-3-3V15a3 3 0 013-3z" stroke="currentColor" stroke-width="2"/>
      </svg>
      <p class="empty-text">还没有项目，打开目录开始吧</p>
      <button class="open-btn small" @click="handleOpenDirectory">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h4l1-1h6a1 1 0 011 1v8a1 1 0 01-1 1H2a1 1 0 01-1-1V5a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
        </svg>
        打开目录
      </button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.welcome-view {
  padding: 40px 32px;
  max-width: 960px;
  margin: 0 auto;
  overflow-y: auto;
  height: 100%;
}

// ─── Header ───
.welcome-header {
  margin-bottom: 40px;
}

.welcome-logo {
  display: flex;
  align-items: center;
  gap: 16px;
}

.logo-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 14px;
  background: linear-gradient(135deg, var(--primary), var(--accent-purple));
  color: #fff;
  font-size: 18px;
  font-weight: 800;
  flex-shrink: 0;
}

.welcome-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-sans);
}

.welcome-subtitle {
  font-size: 14px;
  color: var(--text-tertiary);
  margin: 4px 0 0 0;
}

// ─── Actions bar ───
.actions-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
  gap: 16px;
}

.open-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 9px 18px;
  border: none;
  background: var(--primary);
  color: #fff;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  font-family: var(--font-sans);

  &:hover {
    background: var(--primary-hover);
    transform: translateY(-1px);
  }

  &:active {
    transform: translateY(0);
  }

  &.small {
    padding: 7px 14px;
    font-size: 12px;
  }
}

.search-wrapper {
  position: relative;
  width: 280px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 8px 12px 8px 34px;
  border: 0.5px solid var(--border-default);
  background: var(--bg-secondary);
  color: var(--text-primary);
  border-radius: 8px;
  font-size: 13px;
  font-family: var(--font-sans);
  outline: none;
  transition: border-color var(--transition-fast);

  &:focus {
    border-color: var(--primary);
  }

  &::placeholder {
    color: var(--text-tertiary);
  }
}

// ─── Section ───
.section {
  margin-bottom: 32px;
}

.section-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-tertiary);
  margin-bottom: 12px;
}

// ─── Projects grid ───
.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 8px;
}

.project-card {
  padding: 14px 16px;
  background: var(--bg-secondary);
  border: 0.5px solid var(--border-default);
  border-radius: 10px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    border-color: var(--primary-light);
    transform: translateY(-1px);
  }
}

.project-card-main {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.project-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-light);
  border-radius: 8px;
  color: var(--primary);
  flex-shrink: 0;
}

.project-info {
  flex: 1;
  min-width: 0;
}

.project-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.project-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.project-path {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
  font-family: var(--font-mono);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.project-delete {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: 6px;
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  flex-shrink: 0;

  .project-card:hover & {
    opacity: 1;
  }

  &:hover {
    background: var(--error-light);
    color: var(--error);
  }
}

.project-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 8px;
  padding-left: 44px;
}

.project-branch {
  font-size: 11px;
  color: var(--accent-teal);
  font-family: var(--font-mono);
  padding: 1px 6px;
  background: var(--success-light);
  border-radius: 4px;
}

.project-time {
  font-size: 11px;
  color: var(--text-tertiary);
}

// ─── Empty state ───
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-tertiary);

  .empty-text {
    margin: 16px 0 20px;
    font-size: 14px;
  }
}

@media (max-width: 767px) {
  .welcome-view {
    padding: 20px 16px;
  }

  .welcome-title {
    font-size: 22px;
  }

  .actions-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .search-wrapper {
    width: 100%;
  }

  .projects-grid {
    grid-template-columns: 1fr;
  }
}
</style>
