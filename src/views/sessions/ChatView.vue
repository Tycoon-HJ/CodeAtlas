<script setup lang="ts">
import { NText, NInput, NSelect, NScrollbar, NPopconfirm } from 'naive-ui'
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSessionStore } from '@/stores/session'
import PermissionDialog from '@/components/common/PermissionDialog.vue'
import TrustPromptDialog from '@/components/common/TrustPromptDialog.vue'
import FileTree from '@/components/common/FileTree.vue'
import CodeEditor from '@/components/common/CodeEditor.vue'
import TerminalPanel from '@/components/common/TerminalPanel.vue'
import { useAgentStore } from '@/stores/agent'
import { useProjectStore } from '@/stores/project'
import { useSettingsStore } from '@/stores/settings'
import { useEditorStore } from '@/stores/editor'
import { marked } from 'marked'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'
import * as api from '@/api'
import type { Message } from '@/types'
import { commandRegistry, parseCommand, type CommandContext, type CommandMeta } from '@/commands/registry'
import '@/commands/handlers' // registers all handlers

const props = defineProps<{ projectId: string; sessionId?: string }>()
const router = useRouter()
const sessionStore = useSessionStore()
const agentStore = useAgentStore()
const projectStore = useProjectStore()
const settingsStore = useSettingsStore()
const editorStore = useEditorStore()
const inputMessage = ref('')
const activeSession = ref<any>(null)
const scrollRef = ref<InstanceType<typeof NScrollbar> | null>(null)
const commandHistory = ref<string[]>([])
const historyIndex = ref<number>(-1)
const showCommandSuggestions = ref(false)
const commandSuggestions = ref<CommandMeta[]>([])
const selectedSuggestionIndex = ref(0)
const showLeftPanel = ref(false)
const showRightPanel = ref(true)
const rightPanelTab = ref<'tree' | 'editor'>('tree')
const showTerminal = ref(false)
const terminalLines = ref<string[]>([])

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

let unlistenStderr: UnlistenFn | null = null
let unlistenOutput: UnlistenFn | null = null

onMounted(async () => {
  await projectStore.fetchProjects()
  await sessionStore.fetchSessions()
  await initializeActiveSession()
  editorStore.initFileChangeListener()

  // Start file watcher for the project
  if (project.value?.path) {
    api.startFileWatcher(project.value.path).catch(console.error)
  }

  unlistenStderr = await listen('claude-stderr', (event: any) => {
    if (event.payload?.message) {
      terminalLines.value.push(`[stderr] ${event.payload.message}`)
    }
  })

  unlistenOutput = await listen('claude-output', (event: any) => {
    if (event.payload?.content && !event.payload.done) {
      terminalLines.value.push(event.payload.content)
    }
  })
})

onBeforeUnmount(() => {
  unlistenStderr?.()
  unlistenOutput?.()
  api.stopFileWatcher().catch(console.error)
})

const project = computed(() =>
  projectStore.projects.find((p) => p.id === props.projectId)
)

// Reinitialize when navigating to a different session
watch(() => props.sessionId, async (newId) => {
  if (newId && newId !== activeSession.value?.id) {
    await initializeActiveSession()
  }
})

const sessionMessages = computed(() =>
  activeSession.value ? sessionStore.messages.filter((m) => m.sessionId === activeSession.value.id) : []
)

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

function goToNewChat() {
  router.push(`/project/${props.projectId}/chat`)
}

// ─── Command suggestion filtering ───

function updateSuggestions() {
  const input = inputMessage.value
  if (input.startsWith('/')) {
    const parsed = parseCommand(input)
    const query = parsed ? parsed.name.substring(1) : input.substring(1)
    const all = commandRegistry.getMetaList()
    const filtered = query.length === 0
      ? all
      : all.filter(
          (cmd) =>
            cmd.name.toLowerCase().includes(query.toLowerCase()) ||
            cmd.description.toLowerCase().includes(query.toLowerCase())
        )
    commandSuggestions.value = filtered
    showCommandSuggestions.value = filtered.length > 0
    selectedSuggestionIndex.value = 0
  } else {
    showCommandSuggestions.value = false
    commandSuggestions.value = []
  }
}

watch(inputMessage, updateSuggestions)

// ─── Build command context ───

function buildCommandContext(): CommandContext {
  return {
    sessionId: activeSession.value?.id ?? '',
    workingDir: sessionContext.value?.workingDirectory ?? project.value?.path ?? '',
    claudePath: settingsStore.settings.claudePath,
    claudeConfigPath: settingsStore.settings.claudeConfigPath,
    messageCount: sessionMessages.value.length,
    projectName: sessionContext.value?.projectName ?? project.value?.name ?? '',
  }
}

// ─── Session initialization ───

async function initializeActiveSession() {
  const proj = projectStore.projects.find((p) => p.id === props.projectId)
  if (!proj) {
    console.error('Project not found:', props.projectId)
    return
  }

  let session
  if (props.sessionId) {
    session = sessionStore.sessions.find((s) => s.id === props.sessionId)
    if (!session) {
      await sessionStore.fetchSessions()
      session = sessionStore.sessions.find((s) => s.id === props.sessionId)
    }
    if (!session) {
      console.error('Session not found:', props.sessionId)
      session = await sessionStore.createSession(props.projectId, selectedProvider.value)
    } else {
      selectedProvider.value = session.providerId
    }
  } else {
    session = await sessionStore.createSession(props.projectId, selectedProvider.value)
  }

  activeSession.value = session
  sessionStore.setCurrentSession(session)
  sessionStore.initSessionContext(session.id, props.projectId, proj.name, proj.path)
  await sessionStore.fetchMessages(session.id)
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

  if (showCommandSuggestions.value && commandSuggestions.value.length > 0) {
    if (event.key === 'ArrowUp') {
      event.preventDefault()
      selectedSuggestionIndex.value =
        selectedSuggestionIndex.value > 0
          ? selectedSuggestionIndex.value - 1
          : commandSuggestions.value.length - 1
      scrollToSelectedSuggestion()
      return
    } else if (event.key === 'ArrowDown') {
      event.preventDefault()
      selectedSuggestionIndex.value =
        selectedSuggestionIndex.value < commandSuggestions.value.length - 1
          ? selectedSuggestionIndex.value + 1
          : 0
      scrollToSelectedSuggestion()
      return
    } else if (event.key === 'Enter' && !event.ctrlKey && !event.metaKey) {
      event.preventDefault()
      selectCommand(commandSuggestions.value[selectedSuggestionIndex.value])
      return
    } else if (event.key === 'Escape') {
      event.preventDefault()
      showCommandSuggestions.value = false
      return
    } else if (event.key === 'Tab') {
      event.preventDefault()
      selectCommand(commandSuggestions.value[selectedSuggestionIndex.value])
      return
    }
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

function scrollToSelectedSuggestion() {
  nextTick(() => {
    const list = document.querySelector('.suggestions-list')
    const item = list?.querySelector('.suggestion-item.active')
    if (item) {
      item.scrollIntoView({ block: 'nearest' })
    }
  })
}

function selectCommand(command: CommandMeta) {
  inputMessage.value = command.name + ' '
  showCommandSuggestions.value = false
  nextTick(() => {
    const input = document.querySelector('.input-editor input, .input-editor textarea') as HTMLElement
    input?.focus()
  })
}

// ─── Send message / execute command ───

async function handleSend() {
  if (!inputMessage.value.trim() || sending.value || !activeSession.value) return

  const content = inputMessage.value.trim()
  commandHistory.value.unshift(content)
  historyIndex.value = -1
  inputMessage.value = ''
  showCommandSuggestions.value = false

  // Check if it's a slash command
  const parsed = parseCommand(content)
  if (parsed) {
    const def = commandRegistry.get(parsed.name)
    if (def) {
      // Add user message showing the command
      const userMsg: Message = {
        id: crypto.randomUUID(),
        sessionId: activeSession.value.id,
        type: 'user',
        content,
        createdAt: new Date().toISOString(),
      }
      await sessionStore.addMessage(userMsg)
      scrollToBottom()

      // Execute command via registry
      const ctx = buildCommandContext()
      const result = await commandRegistry.execute(content, ctx)

      if (!result) return

      if (result.type === 'silent') {
        // No output (e.g., /clear)
        scrollToBottom()
        return
      }

      if (result.type === 'claude' && result.prompt) {
        // Send the translated prompt to Claude
        try {
          sessionStore.markPending(activeSession.value.id)
          await api.sendToClaude(
            activeSession.value.id,
            result.prompt,
            ctx.workingDir,
            ctx.claudePath,
            ctx.claudeConfigPath
          )
        } catch (e: any) {
          sessionStore.clearPending(activeSession.value.id)
          const errMsg: Message = {
            id: crypto.randomUUID(),
            sessionId: activeSession.value.id,
            type: 'system',
            content: '发送失败: ' + (e?.message ?? String(e)),
            createdAt: new Date().toISOString(),
          }
          await sessionStore.addMessage(errMsg)
        }
        return
      }

      // type === 'text' or 'error' — render locally
      const resultMsg: Message = {
        id: crypto.randomUUID(),
        sessionId: activeSession.value.id,
        type: result.type === 'error' ? 'system' : 'assistant',
        content: result.content,
        createdAt: new Date().toISOString(),
      }
      await sessionStore.addMessage(resultMsg)
      scrollToBottom()
      return
    }
  }

  // Not a slash command — send to Claude normally
  const userMsg: Message = {
    id: crypto.randomUUID(),
    sessionId: activeSession.value.id,
    type: 'user',
    content,
    createdAt: new Date().toISOString(),
  }
  await sessionStore.addMessage(userMsg)
  scrollToBottom()

  try {
    sessionStore.markPending(activeSession.value.id)
    const workingDir = sessionContext.value?.workingDirectory ?? project.value?.path ?? ''
    const claudePath = settingsStore.settings.claudePath
    const claudeConfigPath = settingsStore.settings.claudeConfigPath
    await api.sendToClaude(activeSession.value.id, content, workingDir, claudePath, claudeConfigPath)
  } catch (e: any) {
    sessionStore.clearPending(activeSession.value.id)
    const errMsg: Message = {
      id: crypto.randomUUID(),
      sessionId: activeSession.value.id,
      type: 'system',
      content: '发送失败: ' + (e?.message ?? String(e)),
      createdAt: new Date().toISOString(),
    }
    await sessionStore.addMessage(errMsg)
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

// ─── Three-panel helpers ───

const projectSessions = computed(() =>
  sessionStore.sessions.filter((s) => s.taskId === props.projectId)
)

function goToSession(sessionId: string) {
  router.push(`/project/${props.projectId}/chat/${sessionId}`)
}

function handleFileClick(path: string) {
  editorStore.openFile(path)
  rightPanelTab.value = 'editor'
}

function formatTime(iso: string): string {
  const d = new Date(iso)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return '刚刚'
  if (diffMin < 60) return `${diffMin}分钟前`
  const diffH = Math.floor(diffMin / 60)
  if (diffH < 24) return `${diffH}小时前`
  return `${d.getMonth() + 1}/${d.getDate()} ${d.getHours()}:${String(d.getMinutes()).padStart(2, '0')}`
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
            <button class="icon-btn" @click="goToNewChat" title="新建对话">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
            <button class="icon-btn" @click="showRightPanel = !showRightPanel" title="文件树">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M2 3h5l1 1h5a1 1 0 011 1v8a1 1 0 01-1 1H2a1 1 0 01-1-1V4a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
              </svg>
            </button>
            <button class="icon-btn" :class="{ active: showTerminal }" @click="showTerminal = !showTerminal" title="终端">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M2 4l5 4-5 4M9 12h5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
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
            <NText depth="3" style="font-size: 14px">开始与 Claude 对话</NText>
            <NText depth="3" style="font-size: 12px; margin-top: 4px">输入 / 查看可用命令</NText>
          </div>

          <!-- 消息列表 -->
          <div v-for="msg in sessionMessages" :key="msg.id" class="message-row" :class="msg.type">
            <div v-if="msg.type !== 'user'" class="avatar-col">
              <div class="avatar ai">AI</div>
            </div>
            <div class="bubble" :class="msg.type">
              <details v-if="msg.type === 'assistant' && msg.thinking" class="thinking-block">
                <summary class="thinking-label">思考过程</summary>
                <div class="thinking-content markdown-body" v-html="renderMarkdown(msg.thinking)" />
              </details>
              <div
                v-if="msg.type === 'assistant'"
                class="bubble-content markdown-body"
                v-html="renderMarkdown(msg.content)"
              />
              <div v-else class="bubble-content" v-html="renderPlain(msg.content)" />
              <div class="bubble-time">{{ msg.createdAt.slice(11, 16) }}</div>
            </div>
            <div v-if="msg.type === 'user'" class="avatar-col">
              <div class="avatar user">U</div>
            </div>
          </div>

          <!-- 流式响应 -->
          <div v-if="sending" class="message-row assistant">
            <div class="avatar-col">
              <div class="avatar ai">AI</div>
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
              <div v-if="!streamThinking && !streamContent && currentToolCalls.length === 0" class="bubble-content typing">
                <span class="dot"></span><span class="dot"></span><span class="dot"></span>
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
              placeholder="输入消息... (Ctrl+Enter 发送，/ 查看命令)"
              :rows="1"
              :autosize="{ minRows: 1, maxRows: 6 }"
              :disabled="sending"
              @keydown="handleKeydown"
              class="input-editor"
            />

            <!-- 命令补全建议 -->
            <div v-if="showCommandSuggestions && commandSuggestions.length > 0" class="command-suggestions">
              <div class="suggestions-header">
                <span>斜杠命令</span>
                <span class="suggestions-count">{{ commandSuggestions.length }}</span>
              </div>
              <div class="suggestions-list">
                <div
                  v-for="(cmd, i) in commandSuggestions"
                  :key="cmd.name"
                  class="suggestion-item"
                  :class="{ active: i === selectedSuggestionIndex }"
                  @click="selectCommand(cmd)"
                  @mouseenter="selectedSuggestionIndex = i"
                >
                  <div class="suggestion-main">
                    <span class="suggestion-name">{{ cmd.icon }} {{ cmd.name }}</span>
                    <span class="suggestion-usage">{{ cmd.usage }}</span>
                  </div>
                  <span class="suggestion-desc">{{ cmd.description }}</span>
                </div>
              </div>
            </div>
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

        <!-- 终端面板 -->
        <div v-if="showTerminal" class="terminal-area">
          <div class="terminal-header">
            <span class="terminal-title">终端输出</span>
            <div class="terminal-header-actions">
              <button class="icon-btn" @click="terminalLines = []" title="清空">
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                  <path d="M3 3l10 10M13 3L3 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                </svg>
              </button>
              <button class="icon-btn" @click="showTerminal = false" title="关闭">
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                  <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                </svg>
              </button>
            </div>
          </div>
          <TerminalPanel :lines="terminalLines" :active="showTerminal" />
        </div>
      </div>

      <!-- 右侧面板：文件树 + 编辑器 -->
      <div v-if="showRightPanel" class="side-panel right-panel">
        <div class="panel-header">
          <div class="panel-tabs">
            <button
              class="panel-tab"
              :class="{ active: rightPanelTab === 'tree' }"
              @click="rightPanelTab = 'tree'"
            >文件</button>
            <button
              class="panel-tab"
              :class="{ active: rightPanelTab === 'editor' }"
              @click="rightPanelTab = 'editor'"
            >编辑器</button>
          </div>
          <button class="icon-btn" @click="showRightPanel = false" title="关闭">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <!-- 编辑器标签页 -->
        <div v-if="editorStore.openFiles.length > 0" class="editor-tabs">
          <div
            v-for="f in editorStore.openFiles"
            :key="f.path"
            class="editor-tab"
            :class="{ active: f.path === editorStore.activeFilePath }"
            @click="editorStore.setActive(f.path); rightPanelTab = 'editor'"
          >
            <span class="tab-name">{{ f.name }}</span>
            <span v-if="f.modified" class="tab-modified">●</span>
            <button class="tab-close" @click.stop="editorStore.closeFile(f.path)" title="关闭">×</button>
          </div>
        </div>

        <div class="panel-content panel-body">
          <!-- 文件树 -->
          <div v-show="rightPanelTab === 'tree'" class="tree-container">
            <FileTree
              v-if="project?.path"
              :root-path="project.path"
              @file-click="handleFileClick"
            />
            <div v-else class="empty-panel">
              <NText depth="3" style="font-size: 12px">无项目路径</NText>
            </div>
          </div>

          <!-- Monaco 编辑器 -->
          <div v-show="rightPanelTab === 'editor'" class="editor-container">
            <CodeEditor
              v-if="editorStore.activeFile"
              :content="editorStore.activeFile.content"
              :language="editorStore.activeFile.language"
              :read-only="false"
              @update:content="editorStore.updateContent(editorStore.activeFilePath!, $event)"
            />
            <div v-else class="empty-panel">
              <NText depth="3" style="font-size: 12px">点击文件树中的文件打开编辑器</NText>
            </div>
          </div>
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
  background: var(--bg-primary);
}

.chat-welcome {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
}

.welcome-title {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 32px;

  .title-text {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
  }
}

.session-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

// ─── Three-panel layout ───
.three-panel {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.side-panel {
  width: 260px;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 0.5px solid var(--border-color);
  flex-shrink: 0;

  &.right-panel {
    border-right: none;
    border-left: 0.5px solid var(--border-color);
  }
}

.panel-header {
  padding: 10px 12px;
  border-bottom: 0.5px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;

  .panel-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.center-panel {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.session-list-panel {
  padding: 4px 8px;
}

.session-item {
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background var(--transition-fast);
  display: flex;
  flex-direction: column;
  gap: 2px;

  &:hover {
    background: var(--bg-hover);
  }

  &.active {
    background: rgba(10, 132, 255, 0.12);
    outline: 1px solid rgba(10, 132, 255, 0.3);
    outline-offset: -1px;
  }

  .session-item-title {
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-item-time {
    font-size: 11px;
    color: var(--text-tertiary);
  }
}

.empty-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 12px;
  color: var(--text-tertiary);
}

// ─── Panel tabs ───
.panel-tabs {
  display: flex;
  gap: 2px;
}

.panel-tab {
  padding: 3px 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  border-radius: 4px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  &.active {
    background: rgba(10, 132, 255, 0.12);
    color: var(--accent-blue);
  }
}

// ─── Editor tabs ───
.editor-tabs {
  display: flex;
  overflow-x: auto;
  border-bottom: 0.5px solid var(--border-color);
  flex-shrink: 0;
  background: var(--bg-primary);

  &::-webkit-scrollbar {
    height: 0;
  }
}

.editor-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  font-size: 12px;
  color: var(--text-secondary);
  border-right: 0.5px solid var(--border-color);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
  }

  &.active {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-bottom: 2px solid var(--accent-blue);
  }

  .tab-name {
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-modified {
    color: var(--accent-orange);
    font-size: 10px;
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    border-radius: 3px;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;

    &:hover {
      background: rgba(255, 69, 58, 0.15);
      color: var(--accent-red);
    }
  }
}

.panel-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tree-container,
.editor-container {
  flex: 1;
  overflow: hidden;
}

// ─── Header ───
.session-header {
  padding: 10px 16px;
  border-bottom: 0.5px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-secondary);
  flex-shrink: 0;

  .header-info {
    display: flex;
    align-items: center;
    gap: 8px;
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
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 6px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  &.active {
    background: rgba(10, 132, 255, 0.12);
    color: var(--accent-blue);
  }

  &.danger:hover {
    background: rgba(255, 69, 58, 0.12);
    color: var(--accent-red);
  }
}

// ─── Context bar ───
.context-bar {
  padding: 6px 16px;
  background: var(--bg-secondary);
  border-bottom: 0.5px solid var(--border-color);
  display: flex;
  gap: 20px;
  font-size: 12px;
  flex-shrink: 0;

  .context-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .context-label {
    color: var(--text-tertiary);
    font-weight: 500;
  }

  .context-value {
    color: var(--text-secondary);
    font-family: var(--font-mono);
    padding: 1px 6px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 4px;
  }
}

// ─── Messages area ───
.messages-area {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
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
    color: var(--text-tertiary);
  }
}

// ─── Message rows ───
.message-row {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
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
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  color: #fff;

  &.ai {
    background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
  }

  &.user {
    background: var(--accent-green);
  }
}

// ─── Bubbles ───
.bubble {
  max-width: 72%;
  padding: 10px 14px;
  line-height: 1.65;
  font-size: 14px;
  word-break: break-word;

  &.user {
    background: var(--accent-blue);
    color: #fff;
    border-radius: 18px 18px 4px 18px;

    .bubble-time {
      color: rgba(255, 255, 255, 0.6);
    }
  }

  &.assistant {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-radius: 18px 18px 18px 4px;
    border: 0.5px solid var(--border-color);

    .bubble-time {
      color: var(--text-tertiary);
    }
  }

  &.system {
    background: rgba(255, 69, 58, 0.12);
    color: var(--accent-red);
    font-size: 13px;
    max-width: 100%;
    border-radius: 12px;
    border: 0.5px solid rgba(255, 69, 58, 0.2);
    text-align: center;
  }
}

.bubble-content {
  word-break: break-word;
  white-space: pre-wrap;

  &.markdown-body {
    margin: 0;

    :deep(p) {
      margin: 8px 0;

      &:first-child { margin-top: 0; }
      &:last-child { margin-bottom: 0; }
    }

    :deep(code) {
      background: rgba(255, 255, 255, 0.08);
      padding: 2px 6px;
      border-radius: 4px;
      font-family: var(--font-mono);
      font-size: 0.88em;
    }

    :deep(pre) {
      background: var(--bg-primary);
      color: #e1e4e8;
      padding: 14px;
      border-radius: 10px;
      overflow-x: auto;
      margin: 12px 0;
      font-size: 0.85em;
      border: 0.5px solid var(--border-color);

      code {
        background: transparent;
        padding: 0;
        color: inherit;
      }
    }

    :deep(a) {
      color: var(--accent-blue);
      text-decoration: none;

      &:hover { text-decoration: underline; }
    }

    :deep(strong) { font-weight: 600; }
    :deep(em) { font-style: italic; }

    :deep(ul), :deep(ol) {
      margin: 8px 0;
      padding-left: 24px;
    }

    :deep(li) { margin: 4px 0; }

    :deep(blockquote) {
      margin: 8px 0;
      padding: 4px 12px;
      border-left: 3px solid var(--accent-blue);
      color: var(--text-secondary);
    }
  }
}

.bubble-time {
  font-size: 11px;
  margin-top: 4px;
  text-align: right;
}

// ─── Thinking block ───
.thinking-block {
  background: rgba(255, 159, 10, 0.08);
  border-radius: 8px;
  padding: 10px 12px;
  margin-bottom: 10px;
  border-left: 3px solid var(--accent-orange);

  &.streaming {
    border-left-color: var(--accent-orange);
    animation: thinkingPulse 2s ease-in-out infinite;
  }

  .thinking-label {
    font-weight: 600;
    font-size: 0.88em;
    cursor: pointer;
    user-select: none;
    color: var(--accent-orange);
    padding: 0;
    margin: 0 0 6px 0;

    &:hover { opacity: 0.8; }
  }

  .thinking-content {
    font-size: 0.92em;
    color: var(--text-secondary);
  }
}

// ─── Tool calls ───
.tool-calls-log {
  margin-bottom: 8px;
  padding: 6px 0;
  border-bottom: 0.5px solid var(--border-color);

  .tool-call-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .tool-icon {
    color: var(--accent-purple);
    display: flex;
    align-items: center;
  }

  .tool-name {
    font-weight: 600;
    color: var(--accent-purple);
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px 6px;
    background: rgba(191, 90, 242, 0.1);
    border-radius: 4px;
  }

  .tool-args {
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 300px;
  }
}

.abort-area {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 0.5px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.abort-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 0.5px solid rgba(255, 69, 58, 0.3);
  background: rgba(255, 69, 58, 0.1);
  color: var(--accent-red);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: rgba(255, 69, 58, 0.2);
    border-color: rgba(255, 69, 58, 0.5);
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

// ─── Streaming cursor ───
.streaming-text {
  .cursor {
    display: inline-block;
    width: 2px;
    height: 1em;
    background: var(--accent-green);
    margin-left: 2px;
    vertical-align: text-bottom;
    animation: cursorBlink 1s step-end infinite;
  }
}

// ─── Input area ───
.input-area {
  padding: 12px 16px;
  border-top: 0.5px solid var(--border-color);
  background: var(--bg-secondary);
  display: flex;
  gap: 10px;
  align-items: flex-end;
  flex-shrink: 0;

  .input-wrapper {
    flex: 1;
    position: relative;
  }

  .input-editor {
    width: 100%;

    :deep(.n-input) {
      background: var(--bg-primary);
      border-radius: 12px;
    }

    :deep(.n-input__textarea) {
      font-family: var(--font-sans);
      font-size: 14px;
      color: var(--text-primary);
    }

    :deep(.n-input__textarea-el) {
      &::placeholder {
        color: var(--text-tertiary);
      }
    }
  }
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  background: var(--accent-blue);
  color: #fff;
  border-radius: 10px;
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;

  &:hover:not(:disabled) {
    background: #409CFF;
    transform: scale(1.05);
  }

  &:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  &:active:not(:disabled) {
    transform: scale(0.95);
  }
}

// ─── Terminal area ───
.terminal-area {
  height: 260px;
  min-height: 120px;
  display: flex;
  flex-direction: column;
  border-top: 0.5px solid var(--border-color);
  flex-shrink: 0;
}

.terminal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 12px;
  background: var(--bg-secondary);
  border-bottom: 0.5px solid var(--border-color);
  flex-shrink: 0;

  .terminal-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .terminal-header-actions {
    display: flex;
    gap: 2px;
  }
}

// ─── Command suggestions ───
.command-suggestions {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  max-height: 320px;
  overflow-y: auto;
  background: var(--bg-secondary);
  border: 0.5px solid var(--border-color);
  border-bottom: none;
  border-radius: 12px 12px 0 0;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.4);
  z-index: 10;
  margin-bottom: 4px;

  .suggestions-header {
    padding: 8px 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    border-bottom: 0.5px solid var(--border-color);
    position: sticky;
    top: 0;
    background: var(--bg-secondary);
    display: flex;
    align-items: center;
    justify-content: space-between;

    .suggestions-count {
      font-size: 10px;
      padding: 1px 6px;
      background: rgba(10, 132, 255, 0.15);
      color: var(--accent-blue);
      border-radius: 8px;
    }
  }

  .suggestions-list {
    padding: 4px;
  }

  .suggestion-item {
    padding: 8px 10px;
    cursor: pointer;
    transition: background var(--transition-fast);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    gap: 3px;

    &:hover, &.active {
      background: var(--bg-hover);
    }

    &.active {
      outline: 1px solid var(--accent-blue);
      outline-offset: -1px;
    }

    .suggestion-main {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
    }

    .suggestion-name {
      font-weight: 600;
      color: var(--text-primary);
      font-size: 13px;
      font-family: var(--font-mono);
      white-space: nowrap;
    }

    .suggestion-usage {
      font-size: 11px;
      color: var(--text-tertiary);
      font-family: var(--font-mono);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .suggestion-desc {
      color: var(--text-secondary);
      font-size: 12px;
      line-height: 1.3;
    }
  }
}

// ─── Animations ───
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
