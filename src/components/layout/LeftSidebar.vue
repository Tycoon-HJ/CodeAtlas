<script setup lang="ts">
import { h, ref, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NDropdown } from 'naive-ui'
import { useProjectStore } from '@/stores/project'
import { useSessionStore } from '@/stores/session'
import { useAgentStore } from '@/stores/agent'
import FileTree from '@/components/common/FileTree.vue'

const router = useRouter()
const route = useRoute()
const projectStore = useProjectStore()
const sessionStore = useSessionStore()
const agentStore = useAgentStore()

const projectId = computed(() => route.params.projectId as string | undefined)
const project = computed(() =>
  projectId.value ? projectStore.projects.find((p) => p.id === projectId.value) : null
)

const fileTreeExpanded = ref(false)
const contextMenuSessionId = ref<string | null>(null)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const showContextMenu = ref(false)

// Sessions for current project
const projectSessions = computed(() => {
  if (!projectId.value) return []
  return [...sessionStore.sessions]
    .filter((s) => s.taskId === projectId.value)
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
})

function getProviderName(providerId: string): string {
  return agentStore.providers.find((p) => p.id === providerId)?.name ?? 'AI'
}

function getSummary(session: any): string {
  const msgs = sessionStore.messages.filter((m) => m.sessionId === session.id)
  const firstUser = msgs.find((m) => m.type === 'user')
  if (firstUser) {
    return firstUser.content.length > 40 ? firstUser.content.slice(0, 40) + '...' : firstUser.content
  }
  return `会话 ${session.id.slice(0, 8)}`
}

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

// Context menu for sessions
function getSessionContextMenuOptions() {
  return [
    { label: '复制 ID', key: 'copy' },
    { label: '重命名', key: 'rename' },
    { label: '导出 Markdown', key: 'export' },
    { type: 'divider', key: 'd1' },
    { label: '删除', key: 'delete' },
  ]
}

function openSessionContextMenu(e: MouseEvent, sessionId: string) {
  e.preventDefault()
  contextMenuSessionId.value = sessionId
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  showContextMenu.value = true
}

function handleSessionContextMenu(key: string) {
  const sessionId = contextMenuSessionId.value
  if (!sessionId) return
  switch (key) {
    case 'copy':
      navigator.clipboard.writeText(sessionId)
      break
    case 'delete':
      sessionStore.deleteSession(sessionId)
      if (route.name === 'project-session' && route.params.sessionId === sessionId) {
        router.push(`/project/${projectId.value}`)
      }
      break
  }
  showContextMenu.value = false
}

function goToSession(sessionId: string) {
  router.push(`/project/${projectId.value}/chat/${sessionId}`)
}

function goToNewChat() {
  router.push(`/project/${projectId.value}/chat`)
}

const navItems = computed(() => {
  if (!project.value) return []
  return [
    { key: 'workspace', icon: 'house', label: '项目主页', path: `/project/${project.value.id}` },
    { key: 'chat', icon: 'chat', label: 'AI 对话', path: `/project/${project.value.id}/chat` },
  ]
})

function isActiveNav(key: string): boolean {
  const name = route.name as string
  if (key === 'workspace') return name === 'project-workspace'
  if (key === 'chat') return name === 'project-chat' || name === 'project-session'
  return false
}
</script>

<template>
  <div class="sidebar">
    <!-- Header -->
    <div class="sidebar-header" @click="router.push('/')">
      <span class="app-logo">CA</span>
      <span class="app-name">CodeAtlas</span>
    </div>

    <div class="sidebar-scroll">
      <!-- Current project nav -->
      <div v-if="project" class="sidebar-section">
        <div class="section-label">{{ project.name }}</div>

        <div
          v-for="item in navItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: isActiveNav(item.key) }"
          @click="router.push(item.path)"
        >
          <!-- House icon -->
          <svg v-if="item.icon === 'house'" width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 8l6-5.5L14 8M3.5 7V14h3.5v-3h3v3h3.5V7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <!-- Chat icon -->
          <svg v-if="item.icon === 'chat'" width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 3h12a1 1 0 011 1v7a1 1 0 01-1 1H5l-3 3V4a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <span>{{ item.label }}</span>
        </div>
      </div>

      <!-- Session history -->
      <div v-if="project && projectSessions.length > 0" class="sidebar-section">
        <div class="section-header">
          <span class="section-label">历史会话</span>
          <button class="section-action" @click="goToNewChat" title="新建对话">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M7 2v10M2 7h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <div
          v-for="s in projectSessions"
          :key="s.id"
          class="session-item"
          :class="{ active: route.params.sessionId === s.id }"
          @click="goToSession(s.id)"
          @contextmenu="openSessionContextMenu($event, s.id)"
        >
          <div class="session-summary">{{ getSummary(s) }}</div>
          <div class="session-meta">
            <span class="session-provider">{{ getProviderName(s.providerId) }}</span>
            <span class="session-time">{{ formatTime(s.updatedAt) }}</span>
          </div>
        </div>

        <NDropdown
          :show="showContextMenu"
          trigger="manual"
          :x="contextMenuX"
          :y="contextMenuY"
          :options="getSessionContextMenuOptions()"
          @select="handleSessionContextMenu"
          @clickoutside="showContextMenu = false"
        />
      </div>

      <!-- File tree -->
      <div v-if="project" class="sidebar-section">
        <div
          class="section-header clickable"
          @click="fileTreeExpanded = !fileTreeExpanded"
        >
          <span class="section-label">
            <span class="expand-icon">{{ fileTreeExpanded ? '▼' : '▶' }}</span>
            文件
          </span>
        </div>
        <div v-if="fileTreeExpanded" class="file-tree-wrapper">
          <FileTree :root-path="project.path" />
        </div>
      </div>

      <!-- No project selected -->
      <div v-if="!project" class="sidebar-section">
        <div class="nav-item" @click="router.push('/')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 8l6-5.5L14 8M3.5 7V14h3.5v-3h3v3h3.5V7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>首页</span>
        </div>
      </div>

      <!-- Global section -->
      <div class="sidebar-section global-section">
        <div class="section-label">全局</div>
        <div class="nav-item" @click="router.push('/agents')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="5" r="3" stroke="currentColor" stroke-width="1.2"/>
            <path d="M2 14c0-3.3 2.7-6 6-6s6 2.7 6 6" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <span>Agent 管理</span>
        </div>
        <div class="nav-item" @click="router.push('/settings')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="2.5" stroke="currentColor" stroke-width="1.2"/>
            <path d="M8 1v2M8 13v2M1 8h2M13 8h2M2.9 2.9l1.5 1.5M11.6 11.6l1.5 1.5M13.1 2.9l-1.5 1.5M4.4 11.6l-1.5 1.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
          <span>设置</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  cursor: pointer;
  flex-shrink: 0;
  transition: background var(--transition-fast);
  border-bottom: 0.5px solid var(--border-color);

  &:hover {
    background: var(--bg-hover);
  }
}

.app-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
  color: #fff;
  font-size: 11px;
  font-weight: 700;
}

.app-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.sidebar-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.sidebar-section {
  padding: 8px 0;

  &.global-section {
    border-top: 0.5px solid var(--border-color);
    margin-top: auto;
    padding-top: 8px;
  }
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 14px;

  &.clickable {
    cursor: pointer;
    user-select: none;

    &:hover {
      background: var(--bg-hover);
    }
  }
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-tertiary);
  padding: 4px 14px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.section-action {
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
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
}

.expand-icon {
  font-size: 9px;
  width: 10px;
  display: inline-block;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 13px;
  transition: all var(--transition-fast);
  border-radius: 6px;
  margin: 0 6px;

  svg {
    flex-shrink: 0;
  }

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  &.active {
    background: rgba(10, 132, 255, 0.15);
    color: var(--text-primary);

    svg {
      color: var(--accent-blue);
    }
  }
}

// ─── Session items ───
.session-item {
  padding: 8px 14px;
  cursor: pointer;
  transition: background var(--transition-fast);
  border-left: 2px solid transparent;
  margin: 0 6px;
  border-radius: 6px;

  &:hover {
    background: var(--bg-hover);
  }

  &.active {
    background: rgba(10, 132, 255, 0.1);
    border-left-color: var(--accent-blue);
  }
}

.session-summary {
  font-size: 13px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.session-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.session-provider {
  font-size: 11px;
  color: var(--accent-teal);
  font-weight: 500;
}

.session-time {
  font-size: 11px;
  color: var(--text-tertiary);
}

// ─── File tree ───
.file-tree-wrapper {
  max-height: 400px;
  overflow-y: auto;
  padding: 4px 0;
}
</style>
