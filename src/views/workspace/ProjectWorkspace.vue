<script setup lang="ts">
import { NCard, NGrid, NGi, NText, NButton, NSpace, NTag, NSpin, NPopconfirm } from 'naive-ui'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'
import { useSessionStore } from '@/stores/session'
import type { Project } from '@/types'

const props = defineProps<{ projectId: string }>()
const router = useRouter()
const projectStore = useProjectStore()
const sessionStore = useSessionStore()
const loading = ref(true)

onMounted(async () => {
  await projectStore.fetchProjects()
  await sessionStore.fetchSessions()
  const project = projectStore.projects.find((p) => p.id === props.projectId)
  if (project) {
    projectStore.setCurrentProject(project)
    const { invoke } = await import('@tauri-apps/api/core')
    invoke('open_project', { id: project.id }).catch(() => {})
  }
  loading.value = false
})

const project = computed(() =>
  projectStore.projects.find((p) => p.id === props.projectId)
)

const projectSessions = computed(() =>
  sessionStore.sessions.filter((s) => s.taskId === props.projectId)
)

const recentSessions = computed(() =>
  [...projectSessions.value]
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
    .slice(0, 8)
)

function formatTime(dateStr: string): string {
  const d = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins}分钟前`
  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}小时前`
  return `${d.getMonth() + 1}/${d.getDate()}`
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
          <NTag v-if="project.isFavorite" size="small" :bordered="false" style="background: rgba(255, 159, 10, 0.15); color: var(--accent-orange)">收藏</NTag>
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

    <!-- Recent sessions -->
    <div class="section-card">
      <div class="section-header">
        <span class="section-title">最近会话</span>
        <button class="section-action" @click="goToChat">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M7 2v10M2 7h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          新建
        </button>
      </div>

      <div v-if="recentSessions.length === 0" class="empty-section">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none" style="opacity: 0.3">
          <path d="M6 8h20a2 2 0 012 2v12a2 2 0 01-2 2H12l-6 4V10a2 2 0 012-2z" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <NText depth="3" style="font-size: 13px; color: var(--text-tertiary); margin-top: 8px">暂无会话</NText>
      </div>

      <div v-else class="sessions-grid">
        <div
          v-for="s in recentSessions"
          :key="s.id"
          class="session-card"
          @click="goToSession(s.id)"
        >
          <div class="session-card-header">
            <span class="session-card-title">{{ s.title ?? `会话 ${s.id.slice(0, 8)}` }}</span>
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
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-blue);
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
  background: var(--accent-blue);
  color: #fff;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;

  &:hover {
    background: #409CFF;
    transform: translateY(-1px);
  }

  &:active {
    transform: translateY(0);
  }
}

// ─── Section card ───
.section-card {
  background: var(--bg-secondary);
  border: 0.5px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 0.5px solid var(--border-color);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.section-action {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  background: transparent;
  color: var(--accent-blue);
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: rgba(10, 132, 255, 0.1);
  }
}

.empty-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
}

// ─── Sessions grid ───
.sessions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1px;
  background: var(--border-color);
}

.session-card {
  padding: 14px 18px;
  cursor: pointer;
  transition: background var(--transition-fast);
  background: var(--bg-secondary);

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
    background: rgba(255, 69, 58, 0.12);
    color: var(--accent-red);
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

  .sessions-grid {
    grid-template-columns: 1fr;
  }
}
</style>
