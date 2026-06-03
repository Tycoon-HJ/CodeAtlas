<script setup lang="ts">
import { NText, NInput, NSelect, NScrollbar, NPopconfirm } from 'naive-ui'
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/stores/session'
import PermissionDialog from '@/components/common/PermissionDialog.vue'
import TrustPromptDialog from '@/components/common/TrustPromptDialog.vue'
import { useAgentStore } from '@/stores/agent'
import { useProjectStore } from '@/stores/project'
import { useSettingsStore } from '@/stores/settings'
import { marked } from 'marked'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'
import * as api from '@/api'
import type { Message } from '@/types'

const props = defineProps<{ projectId: string; sessionId?: string }>()
const router = useRouter()
const sessionStore = useSessionStore()
const agentStore = useAgentStore()
const projectStore = useProjectStore()
const settingsStore = useSettingsStore()
const inputMessage = ref('')
const activeSession = ref<any>(null)
const scrollRef = ref<InstanceType<typeof NScrollbar> | null>(null)
const commandHistory = ref<string[]>([])
const historyIndex = ref<number>(-1)
const showLeftPanel = ref(false)

// Configure marked with highlight.js
marked.setOptions({
  breaks: true,
  gfm: true,
} as any)

const renderer = new marked.Renderer()
renderer.code = function ({ text, lang }: { text: string; lang?: string }) {
  const language = lang && hljs.getLanguage(lang) ? lang : 'plaintext'
  const highlighted = hljs.highlight(text, { language }).value
  return `<pre class="code-block"><code class="hljs language-${language}">${highlighted}</code></pre>`
}
marked.use({ renderer })

const providerOptions = computed(() =>
  agentStore.providers.filter((p) => p.enabled).map((p) => ({ label: p.name, value: p.id }))
)

const selectedProvider = ref(agentStore.currentProvider?.id ?? 'claude')
const isInitializing = ref(false) // Flag to prevent provider watch during init

const currentProviderInfo = computed(() => {
  const provider = agentStore.providers.find((p) => p.id === selectedProvider.value)
  return provider || { id: 'claude', name: 'Claude Code', type: 'claude' }
})

const providerDisplayName = computed(() => currentProviderInfo.value.name)

const providerAvatarText = computed(() => {
  const name = currentProviderInfo.value.name
  // Get first 2 characters or initials
  if (name.includes(' ')) {
    return name.split(' ').map((w: string) => w[0]).join('').slice(0, 2).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
})

onMounted(async () => {
  await projectStore.fetchProjects()
  await sessionStore.fetchSessions()
  await initializeActiveSession()
})

onBeforeUnmount(() => {
  // Cleanup if needed
})

const project = computed(() =>
  projectStore.projects.find((p) => p.id === props.projectId)
)

// Watch for provider changes and create new session
watch(selectedProvider, async (newProviderId, oldProviderId) => {
  // Skip if initializing (setting provider from existing session)
  if (isInitializing.value) return
  if (newProviderId !== oldProviderId && activeSession.value) {
    // Create a new session with the new provider
    const newSession = await sessionStore.createSession(props.projectId, newProviderId)
    const proj = project.value
    if (proj) {
      sessionStore.initSessionContext(newSession.id, props.projectId, proj.name, proj.path)
    }
    // Navigate to the new session
    router.push(`/project/${props.projectId}/chat/${newSession.id}`)
  }
})

// Reinitialize when navigating to a different session
watch(() => props.sessionId, async (newId, oldId) => {
  if (newId && newId !== oldId) {
    await initializeActiveSession()
  }
})

const sessionMessages = computed(() => {
  if (!activeSession.value) return []
  const filtered = sessionStore.messages.filter((m) => m.sessionId === activeSession.value.id)
  console.log(`[ChatView] sessionMessages for ${activeSession.value.id}:`, filtered.length, 'of', sessionStore.messages.length, 'total')
  return filtered
})

const sending = computed(() =>
  activeSession.value ? sessionStore.isPending(activeSession.value.id) : false
)

const streamThinking = computed(() =>
  activeSession.value ? sessionStore.getStreamingThinking(activeSession.value.id) : ''
)
const streamContent = computed(() =>
  activeSession.value ? sessionStore.getStreamingContent(activeSession.value.id) : ''
)

const sessionContext = computed(() =>
  activeSession.value ? sessionStore.getSessionContext(activeSession.value.id) : null
)

const currentToolCalls = computed(() =>
  activeSession.value ? sessionStore.getToolCalls(activeSession.value.id) : []
)

const pendingPermission = computed(() =>
  activeSession.value ? sessionStore.getPendingPermission(activeSession.value.id) : null
)

const pendingTrustPrompt = computed(() =>
  activeSession.value ? sessionStore.getPendingTrustPrompt(activeSession.value.id) : null
)

watch([sessionMessages, streamThinking, streamContent, currentToolCalls], () => scrollToBottom(), { deep: true })

async function handleAbort() {
  if (!activeSession.value) return
  await api.abortClaude(activeSession.value.id)
  sessionStore.clearPending(activeSession.value.id)
}

async function handlePermission(allow: boolean) {
  if (!activeSession.value) return
  const sid = activeSession.value.id
  try {
    await api.respondPermission(sid, allow)
  } catch (e) {
    console.error('Failed to respond to permission:', e)
  }
  sessionStore.clearPermission(sid)
}

async function handleTrustPrompt(trust: boolean) {
  if (!activeSession.value) return
  const sid = activeSession.value.id
  try {
    await api.respondTrustPrompt(sid, trust)
  } catch (e) {
    console.error('Failed to respond to trust prompt:', e)
  }
  sessionStore.clearTrustPrompt(sid)
}

async function handleDeleteSession() {
  if (!activeSession.value) return
  await sessionStore.deleteSession(activeSession.value.id)
  router.push(`/project/${props.projectId}`)
}

// ─── Session initialization ───

async function initializeActiveSession() {
  const proj = projectStore.projects.find((p) => p.id === props.projectId)
  if (!proj) {
    console.error('Project not found:', props.projectId)
    return
  }

  // Set flag to prevent provider watch from firing
  isInitializing.value = true

  // Always fetch latest sessions first
  await sessionStore.fetchSessions()

  let session
  if (props.sessionId) {
    // Try to find existing session
    session = sessionStore.sessions.find((s) => s.id === props.sessionId)
    if (session) {
      selectedProvider.value = session.providerId
    } else {
      // Session not found — create new temporary session
      console.warn('Session not found, creating new:', props.sessionId)
      session = await sessionStore.createSession(props.projectId, selectedProvider.value, false)
      // Navigate to the new session URL so the URL matches the actual session
      router.replace(`/project/${props.projectId}/chat/${session.id}`)
    }
  } else {
    // Create temporary session (not saved to backend until first message)
    session = await sessionStore.createSession(props.projectId, selectedProvider.value, false)
  }

  activeSession.value = session
  sessionStore.setCurrentSession(session)
  sessionStore.initSessionContext(session.id, props.projectId, proj.name, proj.path)
  await sessionStore.fetchMessages(session.id)
  console.log('[ChatView] Initialized session:', session.id, 'messages:', sessionStore.messages.length)

  // Reset flag
  isInitializing.value = false
}

function scrollToBottom() {
  nextTick(() => {
    if (scrollRef.value) {
      scrollRef.value.scrollTo({ top: 999999 })
    }
  })
}

// ─── Keyboard handling ───

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    handleSend()
    return
  }

  // Command history navigation
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    if (historyIndex.value < commandHistory.value.length - 1) {
      historyIndex.value++
      inputMessage.value = commandHistory.value[historyIndex.value]
    }
  } else if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (historyIndex.value > 0) {
      historyIndex.value--
      inputMessage.value = commandHistory.value[historyIndex.value]
    } else if (historyIndex.value === 0) {
      historyIndex.value = -1
      inputMessage.value = ''
    }
  }
}

// ─── Send message ───

async function handleSend() {
  if (!inputMessage.value.trim() || sending.value || !activeSession.value) return

  const content = inputMessage.value.trim()
  commandHistory.value.unshift(content)
  historyIndex.value = -1
  inputMessage.value = ''

  // Save session to backend on first message
  const session = activeSession.value
  if (session) {
    // Check if this session has any messages
    const existingMessages = sessionStore.messages.filter(m => m.sessionId === session.id)
    if (existingMessages.length === 0) {
      // This is a new session that hasn't been saved yet
      await sessionStore.saveSessionToBackend(session)
    }
  }

  const userMsg: Message = {
    id: crypto.randomUUID(),
    sessionId: session.id,
    type: 'user',
    content,
    createdAt: new Date().toISOString(),
  }
  sessionStore.addMessage(userMsg)
  scrollToBottom()

  try {
    sessionStore.markPending(session.id)
    // Set session title from first user message
    if (!session.title) {
      const titlePreview = content.length > 50 ? content.slice(0, 50) + '...' : content
      try {
        await sessionStore.updateSessionTitle(session.id, titlePreview)
      } catch (titleErr) {
        console.warn('Failed to update session title:', titleErr)
      }
    }
    const workingDir = sessionContext.value?.workingDirectory ?? project.value?.path ?? ''
    const providerId = selectedProvider.value
    // Use provider-specific paths
    const providerPath = providerId === 'codex' ? settingsStore.settings.codexPath : settingsStore.settings.claudePath
    const providerConfigPath = providerId === 'codex' ? settingsStore.settings.codexConfigPath : settingsStore.settings.claudeConfigPath
    await api.sendMessage(providerId, session.id, content, workingDir, providerPath, providerConfigPath)
  } catch (e: any) {
    sessionStore.clearPending(session.id)
    const errMsg: Message = {
      id: crypto.randomUUID(),
      sessionId: session.id,
      type: 'system',
      content: '发送失败: ' + (e?.message ?? String(e)),
      createdAt: new Date().toISOString(),
    }
    sessionStore.addMessage(errMsg)
  }
}

function renderMarkdown(content: string): string {
  return marked.parse(content) as string
}

function renderPlain(content: string): string {
  return content
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\n/g, '<br>')
}

function formatToolInput(input: Record<string, unknown>): string {
  if (!input || Object.keys(input).length === 0) return ''
  const entries = Object.entries(input)
  if (entries.length === 1 && typeof entries[0][1] === 'string') {
    return entries[0][1]
  }
  return JSON.stringify(input, null, 2)
}

function formatDateTime(isoStr: string): string {
  const d = new Date(isoStr)
  const year = d.getFullYear()
  const month = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const hours = String(d.getHours()).padStart(2, '0')
  const minutes = String(d.getMinutes()).padStart(2, '0')
  const seconds = String(d.getSeconds()).padStart(2, '0')
  return `${year}/${month}/${day} ${hours}:${minutes}:${seconds}`
}

// ─── Three-panel helpers ───

const projectSessions = computed(() =>
  sessionStore.sessions
    .filter((s) => s.taskId === props.projectId)
    .sort((a, b) => (b.updatedAt || '').localeCompare(a.updatedAt || ''))
)

function goToSession(sessionId: string) {
  router.push(`/project/${props.projectId}/chat/${sessionId}`)
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
</script>

<template>
  <div class="chat-view">
    <!-- 如果没有活跃会话，显示初始化界面 -->
    <div v-if="!activeSession" class="chat-welcome">
      <div class="welcome-title">
        <NText class="title-text">{{ project?.name ?? '项目' }}</NText>
        <NText depth="3" class="subtitle-text">初始化会话中...</NText>
      </div>
    </div>

    <!-- 三栏布局 -->
    <div v-else class="three-panel">
      <!-- 左侧面板：会话列表 -->
      <div v-if="showLeftPanel" class="side-panel left-panel">
        <div class="panel-header">
          <span class="panel-title">会话列表</span>
          <button class="icon-btn" @click="showLeftPanel = false" title="关闭">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
        <div class="panel-content session-list-panel">
          <div
            v-for="s in projectSessions"
            :key="s.id"
            class="session-item"
            :class="{ active: s.id === activeSession?.id }"
            @click="goToSession(s.id)"
          >
            <span class="session-item-title">{{ s.title ?? `会话 ${s.id.slice(0, 8)}` }}</span>
            <span class="session-item-time">{{ formatTime(s.updatedAt) }}</span>
          </div>
          <div v-if="projectSessions.length === 0" class="empty-panel">
            <NText depth="3" style="font-size: 12px">暂无会话</NText>
          </div>
        </div>
      </div>

      <!-- 中间面板：聊天区域 -->
      <div class="center-panel">
        <!-- 头部 -->
        <div class="session-header">
          <div class="header-info">
            <button class="icon-btn" @click="showLeftPanel = !showLeftPanel" title="会话列表">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
            <NText strong style="color: var(--text-primary)">{{ project?.name }}</NText>
            <NText depth="3" style="margin-left: 12px; font-size: 12px">会话 {{ activeSession.id.slice(0, 8) }}</NText>
          </div>
          <div class="header-controls">
            <NSelect v-model:value="selectedProvider" :options="providerOptions" size="small" style="width: 140px" />
            <NPopconfirm @positive-click="handleDeleteSession">
              <template #trigger>
                <button class="icon-btn danger" title="删除会话">
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M3 4h10M6 4V3a1 1 0 011-1h2a1 1 0 011 1v1M5 4v8a1 1 0 001 1h4a1 1 0 001-1V4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
              </template>
              确定删除此会话及所有消息？
            </NPopconfirm>
          </div>
        </div>

        <!-- 上下文信息栏 -->
        <div v-if="sessionContext" class="context-bar">
          <span class="context-item">
            <span class="context-label">项目</span>
            <span class="context-value">{{ sessionContext.projectName }}</span>
          </span>
          <span class="context-item">
            <span class="context-label">目录</span>
            <span class="context-value">{{ sessionContext.workingDirectory }}</span>
          </span>
        </div>

        <!-- 消息区域 -->
        <NScrollbar ref="scrollRef" class="messages-area">
          <div v-if="sessionMessages.length === 0 && !sending" class="empty-chat">
            <div class="empty-icon">
              <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
                <path d="M8 12h32a4 4 0 014 4v20a4 4 0 01-4 4H16l-8 8V16a4 4 0 014-4z" stroke="currentColor" stroke-width="2" opacity="0.3"/>
                <circle cx="18" cy="26" r="2" fill="currentColor" opacity="0.3"/>
                <circle cx="24" cy="26" r="2" fill="currentColor" opacity="0.3"/>
                <circle cx="30" cy="26" r="2" fill="currentColor" opacity="0.3"/>
              </svg>
            </div>
            <NText depth="3" style="font-size: 14px">开始与 {{ providerDisplayName }} 对话</NText>
            <NText depth="3" style="font-size: 12px; margin-top: 4px">输入消息开始聊天</NText>
          </div>

          <!-- 消息列表 -->
          <template v-for="msg in sessionMessages" :key="msg.id">
            <!-- System/Error messages -->
            <div v-if="msg.type === 'system'" class="message-row system">
              <div class="bubble system">
                <div class="bubble-content">
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="none" style="display: inline-block; vertical-align: -2px; margin-right: 6px">
                    <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.2"/>
                    <path d="M8 4v5M8 11v1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                  {{ msg.content }}
                </div>
                <div class="bubble-time">{{ formatDateTime(msg.createdAt) }}</div>
              </div>
            </div>
            <!-- User messages -->
            <div v-else-if="msg.type === 'user'" class="message-row user">
              <div class="bubble user">
                <div class="bubble-content" v-html="renderPlain(msg.content)" />
                <div class="bubble-time">{{ formatDateTime(msg.createdAt) }}</div>
              </div>
              <div class="avatar-col">
                <div class="avatar user">U</div>
              </div>
            </div>
            <!-- Assistant messages -->
            <div v-else class="message-row assistant">
              <div class="avatar-col">
                <div class="avatar ai">{{ providerAvatarText }}</div>
              </div>
              <div class="bubble assistant">
                <details v-if="msg.thinking" class="thinking-block">
                  <summary class="thinking-label">思考过程</summary>
                  <div class="thinking-content markdown-body" v-html="renderMarkdown(msg.thinking)" />
                </details>
                <div v-if="msg.metadata?.toolCalls && msg.metadata.toolCalls.length > 0" class="tool-calls-log">
                  <div v-for="(tc, idx) in msg.metadata.toolCalls" :key="idx" class="tool-call-item">
                    <span class="tool-icon">
                      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                        <path d="M8 2v4l3 3M8 2a6 6 0 100 12 6 6 0 000-12z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
                      </svg>
                    </span>
                    <span class="tool-name">{{ tc.toolName }}</span>
                    <span v-if="formatToolInput(tc.toolInput)" class="tool-args">{{ formatToolInput(tc.toolInput) }}</span>
                  </div>
                </div>
                <div class="bubble-content markdown-body" v-html="renderMarkdown(msg.content)" />
                <div class="bubble-time">{{ formatDateTime(msg.createdAt) }}</div>
              </div>
            </div>
          </template>

          <!-- 流式响应 -->
          <div v-if="sending" class="message-row assistant">
            <div class="avatar-col">
              <div class="avatar ai">{{ providerAvatarText }}</div>
            </div>
            <div class="bubble assistant">
              <div v-if="streamThinking" class="thinking-block streaming">
                <div class="thinking-label">思考中...</div>
                <div class="thinking-content markdown-body" v-html="renderMarkdown(streamThinking)" />
              </div>
              <div v-if="currentToolCalls.length > 0" class="tool-calls-log">
                <div v-for="tc in currentToolCalls" :key="tc.toolId" class="tool-call-item">
                  <span class="tool-icon">
                    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                      <path d="M8 2v4l3 3M8 2a6 6 0 100 12 6 6 0 000-12z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                  </span>
                  <span class="tool-name">{{ tc.toolName }}</span>
                  <span v-if="formatToolInput(tc.toolInput)" class="tool-args">{{ formatToolInput(tc.toolInput) }}</span>
                </div>
              </div>
              <div v-if="streamContent" class="bubble-content markdown-body streaming-text">
                <span v-html="renderMarkdown(streamContent)" /><span class="cursor"></span>
              </div>
              <div v-if="!streamThinking && !streamContent && currentToolCalls.length === 0" class="loading-indicator">
                <div class="loading-spinner-small"></div>
                <span class="loading-text">{{ providerDisplayName }} 思考中...</span>
              </div>
              <div class="abort-area">
                <button class="abort-btn" @click="handleAbort">
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                    <rect x="3" y="3" width="10" height="10" rx="2" fill="currentColor"/>
                  </svg>
                  终止
                </button>
              </div>
            </div>
          </div>
        </NScrollbar>

        <!-- 输入区域 -->
        <div class="input-area">
          <div class="input-wrapper">
            <NInput
              v-model:value="inputMessage"
              type="textarea"
              placeholder="输入消息... (Ctrl+Enter 发送)"
              :rows="1"
              :autosize="{ minRows: 1, maxRows: 6 }"
              :disabled="sending"
              @keydown="handleKeydown"
              class="input-editor"
            />
          </div>
          <button
            class="send-btn"
            :disabled="!inputMessage.trim() || sending"
            @click="handleSend"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M2 8l12-5-5 12-2-5-5-2z" fill="currentColor"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Permission dialog -->
    <PermissionDialog
      :show="!!pendingPermission"
      :request="pendingPermission"
      @allow="handlePermission(true)"
      @deny="handlePermission(false)"
    />

    <!-- Trust prompt dialog -->
    <TrustPromptDialog
      :show="!!pendingTrustPrompt"
      :message="pendingTrustPrompt?.message ?? ''"
      @trust="handleTrustPrompt(true)"
      @deny="handleTrustPrompt(false)"
    />
  </div>
</template>

<style scoped lang="scss">
.chat-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  font-family: var(--font-sans);
  color: var(--text-primary);
}

.chat-welcome {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
  background: var(--bg-surface);
}

.welcome-title {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 32px;

  .title-text {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--text-primary);
  }
}

// ─── Three-panel layout ───
.three-panel {
  display: flex;
  height: 100%;
  overflow: hidden;
  background: var(--bg-surface);
}

.side-panel {
  width: 280px;
  min-width: 240px;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-default);
  flex-shrink: 0;
}

.panel-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;

  .panel-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.center-panel {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-surface);
}

.session-list-panel {
  padding: 8px;
}

.empty-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.session-item {
  padding: 10px 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
  display: flex;
  flex-direction: column;
  gap: 4px;

  &:hover {
    background: var(--bg-hover);
  }

  &.active {
    background: var(--primary-light);
    border-left: 2px solid var(--primary);
  }

  .session-item-title {
    font-size: var(--text-base);
    color: var(--text-primary);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-item-time {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }
}

// ─── Header ───
.session-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-surface);
  flex-shrink: 0;

  .header-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  &.active {
    background: var(--primary-light);
    color: var(--primary);
  }

  &.danger:hover {
    background: var(--error-light);
    color: var(--error);
  }
}

// ─── Context bar ───
.context-bar {
  padding: 8px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-default);
  display: flex;
  gap: 24px;
  font-size: var(--text-xs);
  flex-shrink: 0;

  .context-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .context-label {
    color: var(--text-tertiary);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .context-value {
    color: var(--text-secondary);
    font-family: var(--font-mono);
    padding: 2px 8px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
  }
}

// ─── Messages area ───
.messages-area {
  flex: 1;
  padding: 20px 16px;
  overflow-y: auto;
  line-height: 1.6;
}

.empty-chat {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  color: var(--text-tertiary);

  .empty-icon {
    margin-bottom: 16px;
    color: var(--text-disabled);
  }
}

// ─── Message rows ───
.message-row {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  animation: slideUp 0.3s ease-out;

  &.user {
    justify-content: flex-end;
  }

  &.assistant {
    justify-content: flex-start;
  }

  &.system {
    justify-content: center;
  }

  .avatar-col {
    flex-shrink: 0;
    margin-top: 2px;
  }
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-xs);
  font-weight: 700;
  color: #fff;

  &.ai {
    background: var(--primary);
  }

  &.user {
    background: var(--accent-teal);
  }
}

// ─── Bubbles ───
.bubble {
  max-width: 75%;
  padding: 12px 16px;
  line-height: 1.6;
  font-size: var(--text-md);
  word-break: break-word;
  overflow-wrap: break-word;

  &.user {
    background: var(--primary);
    color: #fff;
    border-radius: var(--radius-lg) var(--radius-lg) var(--radius-sm) var(--radius-lg);

    .bubble-time {
      color: rgba(255, 255, 255, 0.7);
    }
  }

  &.assistant {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-radius: var(--radius-lg) var(--radius-lg) var(--radius-lg) var(--radius-sm);
    border: 1px solid var(--border-default);

    .bubble-time {
      color: var(--text-tertiary);
    }
  }

  &.system {
    background: var(--bg-hover);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    max-width: 80%;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
    text-align: left;

    .bubble-time {
      color: var(--text-tertiary);
    }
  }
}

.bubble-content {
  word-break: break-word;
  white-space: pre-wrap;
  overflow-wrap: break-word;

  &.markdown-body {
    margin: 0;

    :deep(p) {
      margin: 8px 0;

      &:first-child { margin-top: 0; }
      &:last-child { margin-bottom: 0; }
    }

    :deep(code) {
      background: var(--bg-hover);
      padding: 2px 6px;
      border-radius: var(--radius-sm);
      font-family: var(--font-mono);
      font-size: 0.9em;
    }

    :deep(pre) {
      background: var(--bg-primary);
      color: var(--text-primary);
      padding: 12px;
      border-radius: var(--radius-md);
      margin: 12px 0;
      font-size: 0.85em;
      border: 1px solid var(--border-default);
      overflow-x: auto;
      white-space: pre-wrap;
      word-wrap: break-word;

      code {
        background: transparent;
        padding: 0;
        color: inherit;
      }
    }

    :deep(a) {
      color: var(--primary);
      text-decoration: none;

      &:hover { text-decoration: underline; }
    }

    :deep(strong) { font-weight: 600; }
    :deep(em) { font-style: italic; }

    :deep(ul), :deep(ol) {
      margin: 8px 0;
      padding-left: 24px;
    }

    :deep(li) {
      margin: 4px 0;
      line-height: 1.6;
    }

    :deep(blockquote) {
      margin: 8px 0;
      padding: 4px 12px;
      border-left: 3px solid var(--primary);
      color: var(--text-secondary);
      background: var(--primary-lighter);
      border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    }
  }
}

.bubble-time {
  font-size: var(--text-xs);
  margin-top: 6px;
  text-align: right;
  color: var(--text-tertiary);
}

// ─── Thinking block ───
.thinking-block {
  background: var(--bg-hover);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  margin-bottom: 10px;
  border-left: 3px solid var(--border-default);

  &.streaming {
    border-left-color: var(--primary);
    animation: thinkingPulse 2s ease-in-out infinite;
  }

  .thinking-label {
    font-weight: 600;
    font-size: var(--text-sm);
    cursor: pointer;
    user-select: none;
    color: var(--text-secondary);
    padding: 0;
    margin: 0 0 6px 0;

    &:hover { opacity: 0.8; }
  }

  .thinking-content {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
  }
}

// ─── Tool calls ───
.tool-calls-log {
  margin-bottom: 10px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-default);

  .tool-call-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    font-size: var(--text-sm);
    color: var(--text-tertiary);
  }

  .tool-icon {
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
  }

  .tool-name {
    font-weight: 600;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 2px 8px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
  }

  .tool-args {
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 300px;
  }
}

.abort-area {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--border-default);
  display: flex;
  justify-content: flex-end;
}

.abort-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--border-default);
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
}

// ─── Typing dots ───
.typing {
  display: flex;
  gap: 4px;
  padding: 4px 0;

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-tertiary);
    animation: dotPulse 1.4s ease-in-out infinite;

    &:nth-child(2) { animation-delay: 0.2s; }
    &:nth-child(3) { animation-delay: 0.4s; }
  }
}

// ─── Loading indicator ───
.loading-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 0;
}

.loading-spinner-small {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-default);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

.loading-text {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  font-style: italic;
}

// ─── Streaming cursor ───
.streaming-text {
  .cursor {
    display: inline-block;
    width: 2px;
    height: 1em;
    background: var(--primary);
    margin-left: 2px;
    vertical-align: text-bottom;
    animation: cursorBlink 1s step-end infinite;
  }
}

// ─── Input area ───
.input-area {
  padding: 16px;
  border-top: 1px solid var(--border-default);
  background: var(--bg-surface);
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-shrink: 0;

  .input-wrapper {
    flex: 1;
    position: relative;
  }

  .input-editor {
    width: 100%;

    :deep(.n-input) {
      background: var(--bg-surface);
      border-radius: var(--radius-lg);
      border: 1px solid var(--border-default);
      transition: all var(--transition-fast);

      &:hover {
        border-color: var(--text-disabled);
      }

      &.n-input--focus {
        border-color: var(--primary);
        box-shadow: 0 0 0 3px var(--primary-light);
      }
    }

    :deep(.n-input__textarea) {
      font-family: var(--font-sans);
      font-size: var(--text-md);
      color: var(--text-primary);
    }

    :deep(.n-input__textarea-el) {
      &::placeholder {
        color: var(--text-disabled);
      }
    }
  }
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: none;
  background: var(--primary);
  color: #fff;
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;

  &:hover:not(:disabled) {
    background: var(--primary-hover);
    transform: scale(1.05);
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  &:active:not(:disabled) {
    transform: scale(0.95);
  }
}

// ─── Animations ───
@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes cursorBlink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

@keyframes dotPulse {
  0%, 80%, 100% { opacity: 0.2; transform: scale(0.8); }
  40% { opacity: 1; transform: scale(1); }
}

@keyframes thinkingPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}
</style>
