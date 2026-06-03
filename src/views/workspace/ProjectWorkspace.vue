<script setup lang="ts">
import { NCard, NGrid, NGi, NText, NButton, NSpace, NTag, NSpin, NPopconfirm } from 'naive-ui'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'
import { useSessionStore } from '@/stores/session'
import * as api from '@/api'
import type { Project, Message } from '@/types'

const props = defineProps<{ projectId: string }>()
const router = useRouter()
const projectStore = useProjectStore()
const sessionStore = useSessionStore()
const loading = ref(true)
const searchQuery = ref('')
const sessionFirstMessages = ref<Record<string, string>>({})

onMounted(async () => {
  await projectStore.fetchProjects()
  await sessionStore.fetchSessions()
  const project = projectStore.projects.find((p) => p.id === props.projectId)
  if (project) {
    projectStore.setCurrentProject(project)
    const { invoke } = await import('@tauri-apps/api/core')
    invoke('open_project', { id: project.id }).catch(() => {})
  }
  // Load first message for each session
  await loadSessionFirstMessages()
  loading.value = false
})

const project = computed(() =>
  projectStore.projects.find((p) => p.id === props.projectId)
)

const projectSessions = computed(() =>
  sessionStore.sessions.filter((s) => s.taskId === props.projectId)
)

async function loadSessionFirstMessages() {
  const sessions = projectSessions.value
  const results: Record<string, string> = {}
  for (const session of sessions) {
    try {
      const messages = await api.listMessages(session.id)
      const firstUserMsg = messages.find((m: any) => m.type === 'user')
      if (firstUserMsg) {
        const content = firstUserMsg.content as string
        results[session.id] = content.length > 50 ? content.slice(0, 50) + '...' : content
      }
    } catch (e) {
      console.error(`Failed to load messages for session ${session.id}:`, e)
    }
  }
  sessionFirstMessages.value = results
}

const filteredSessions = computed(() => {
  const q = searchQuery.value.toLowerCase()
  let list = [...projectSessions.value]
    .filter((s) => sessionFirstMessages.value[s.id]) // Only show sessions with messages
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  if (q) {
    list = list.filter((s) => {
      const title = sessionFirstMessages.value[s.id] ?? ''
      return title.toLowerCase().includes(q)
    })
  }
  return list
})

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

function goToChat() {
  router.push(`/project/${props.projectId}/chat`)
}

function goToSession(sessionId: string) {
  router.push(`/project/${props.projectId}/chat/${sessionId}`)
}

async function deleteSession(sessionId: string) {
  await sessionStore.deleteSession(sessionId)
}
</script>

<template>
  <div v-if="loading" class="loading">
    <div class="loading-spinner"></div>
  </div>
  <div v-else-if="!project" class="not-found">
    <NText depth="3" style="color: var(--text-secondary)">项目未找到</NText>
    <NButton @click="router.push('/')" style="margin-top: 12px">返回首页</NButton>
  </div>
  <div v-else class="project-workspace">
    <!-- Project header -->
    <div class="project-header">
      <div class="header-main">
        <div class="project-title-row">
          <h1 class="project-title">{{ project.name }}</h1>
          <NTag v-if="project.isFavorite" size="small" :bordered="false" style="background: var(--warning-light); color: var(--warning)">收藏</NTag>
        </div>
        <div class="project-path">{{ project.path }}</div>
        <div v-if="project.description" class="project-desc">{{ project.description }}</div>
      </div>
      <button class="new-chat-btn" @click="goToChat">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        新建对话
      </button>
    </div>

    <!-- Sessions -->
    <div class="section-card">
      <div class="section-header">
        <span class="section-title">会话列表</span>
        <div class="header-actions">
          <div class="search-wrapper">
            <svg class="search-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
              <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.2"/>
              <path d="M11 11l3 3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            </svg>
            <input
              v-model="searchQuery"
              class="search-input"
              placeholder="搜索会话..."
            />
          </div>
          <button class="section-action" @click="goToChat">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M7 2v10M2 7h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            新建
          </button>
        </div>
      </div>

      <div v-if="filteredSessions.length === 0" class="empty-section">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none" style="opacity: 0.3">
          <path d="M6 8h20a2 2 0 012 2v12a2 2 0 01-2 2H12l-6 4V10a2 2 0 012-2z" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <NText depth="3" style="font-size: 13px; color: var(--text-tertiary); margin-top: 8px">
          {{ searchQuery ? '没有匹配的会话' : '暂无会话' }}
        </NText>
      </div>

      <div v-else class="sessions-list">
        <div
          v-for="s in filteredSessions"
          :key="s.id"
          class="session-card"
          @click="goToSession(s.id)"
        >
          <div class="session-card-header">
            <span class="session-card-title">{{ sessionFirstMessages[s.id] }}</span>
            <NPopconfirm @positive-click="deleteSession(s.id)">
              <template #trigger>
                <button class="session-delete" @click.stop title="删除">
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                    <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                </button>
              </template>
              确定删除此会话及所有消息？
            </NPopconfirm>
          </div>
          <div class="session-card-meta">
            <span class="session-card-time">{{ formatTime(s.updatedAt) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.project-workspace {
  padding: 32px;
  max-width: 960px;
  margin: 0 auto;
  overflow-y: auto;
  height: 100%;
}

.loading, .not-found {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-default);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

// ─── Header ───
.project-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 32px;
  gap: 16px;
}

.header-main {
  flex: 1;
  min-width: 0;
}

.project-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.project-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-sans);
}

.project-path {
  font-size: 13px;
  margin-top: 6px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.project-desc {
  font-size: 14px;
  margin-top: 8px;
  color: var(--text-secondary);
}

.new-chat-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  background: var(--primary);
  color: #fff;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;

  &:hover {
    background: var(--primary-hover);
    transform: translateY(-1px);
  }

  &:active {
    transform: translateY(0);
  }
}

// ─── Section card ───
.section-card {
  background: var(--bg-secondary);
  border: 0.5px solid var(--border-default);
  border-radius: 12px;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-default);
  gap: 12px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  flex-shrink: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-wrapper {
  position: relative;
  width: 200px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 6px 10px 6px 30px;
  border: 1px solid var(--border-default);
  background: var(--bg-surface);
  color: var(--text-primary);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  font-family: var(--font-sans);
  outline: none;
  transition: border-color var(--transition-fast);

  &:focus {
    border-color: var(--primary);
  }

  &::placeholder {
    color: var(--text-disabled);
  }
}

.section-action {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  background: transparent;
  color: var(--primary);
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--primary-light);
  }
}

.empty-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
}

// ─── Sessions list ───
.sessions-list {
  display: flex;
  flex-direction: column;
}

.session-card {
  padding: 14px 18px;
  cursor: pointer;
  transition: background var(--transition-fast);
  border-bottom: 1px solid var(--border-default);

  &:last-child {
    border-bottom: none;
  }

  &:hover {
    background: var(--bg-hover);
  }
}

.session-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.session-card-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.session-delete {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: 4px;
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  flex-shrink: 0;

  .session-card:hover & {
    opacity: 1;
  }

  &:hover {
    background: var(--error-light);
    color: var(--error);
  }
}

.session-card-meta {
  margin-top: 6px;
}

.session-card-time {
  font-size: 11px;
  color: var(--text-tertiary);
}

@media (max-width: 767px) {
  .project-workspace {
    padding: 16px 12px;
  }

  .project-header {
    flex-direction: column;
  }

  .project-title {
    font-size: 20px;
  }

  .section-header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-actions {
    width: 100%;
  }

  .search-wrapper {
    width: 100%;
    flex: 1;
  }
}
</style>
