import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Session, Message, SessionStatus } from '@/types'
import * as api from '@/api'

// 会话上下文
export interface SessionContext {
  workingDirectory: string // 当前工作目录
  projectId: string // 项目 ID
  projectName: string // 项目名称
  projectPath: string // 项目路径
}

// 工具调用记录
export interface ToolCall {
  sessionId: string
  toolId: string
  toolName: string
  toolInput: Record<string, unknown>
  toolNumber: number
  timestamp: string
}

export const useSessionStore = defineStore('session', () => {
  const sessions = ref<Session[]>([])
  const currentSession = ref<Session | null>(null)
  const messages = ref<Message[]>([])
  const loading = ref(false)
  const pendingResponses = ref<Set<string>>(new Set())
  const streamingThinking = ref<Record<string, string>>({})
  const streamingContent = ref<Record<string, string>>({})
  const sessionContexts = ref<Record<string, SessionContext>>({})
  const toolCalls = ref<Record<string, ToolCall[]>>({})
  const pendingPermissions = ref<Record<string, any>>({})
  const pendingTrustPrompts = ref<Record<string, any>>({})
  let unlistenOutput: UnlistenFn | null = null
  let unlistenThinking: UnlistenFn | null = null
  let unlistenToolCall: UnlistenFn | null = null
  let unlistenStderr: UnlistenFn | null = null
  let unlistenPermission: UnlistenFn | null = null

  async function initClaudeListener() {
    if (unlistenOutput) return

    // Listen for tool calls (display only, no confirmation needed)
    unlistenToolCall = await listen('claude-tool-call', (event: any) => {
      const payload = event.payload
      if (!payload) return
      const sid = payload.session_id
      if (!toolCalls.value[sid]) {
        toolCalls.value[sid] = []
      }
      toolCalls.value[sid].push({
        sessionId: sid,
        toolId: payload.tool_id,
        toolName: payload.tool_name,
        toolInput: payload.tool_input || {},
        toolNumber: payload.tool_number,
        timestamp: new Date().toISOString(),
      })
      console.log(`[Tool Call] ${payload.tool_name}:`, payload.tool_input)
    })

    // Listen for stderr output
    unlistenStderr = await listen('claude-stderr', (event: any) => {
      const payload = event.payload
      if (!payload) return
      console.warn(`[Claude stderr] ${payload.message}`)
    })

    unlistenThinking = await listen('claude-thinking', (event: any) => {
      const payload = event.payload
      if (!payload || payload.done) return
      streamingThinking.value[payload.session_id] = payload.thinking || ''
    })

    unlistenOutput = await listen('claude-output', async (event: any) => {
      const payload = event.payload
      if (!payload) return

      const sid = payload.session_id

      if (payload.done) {
        // Replace Set to trigger Vue reactivity (Set.delete is not reactive)
        const next = new Set(pendingResponses.value)
        next.delete(sid)
        pendingResponses.value = next

        const thinking = streamingThinking.value[sid] || payload.thinking || ''
        const aiMsg: Message = {
          id: crypto.randomUUID(),
          sessionId: sid,
          type: 'assistant',
          content: payload.content || 'Claude CLI 未返回内容',
          thinking: thinking || undefined,
          createdAt: new Date().toISOString(),
        }
        await api.createMessage(aiMsg.id, aiMsg.sessionId, aiMsg.type, aiMsg.content, aiMsg.createdAt, aiMsg.thinking)
        messages.value.push(aiMsg)
        delete streamingThinking.value[sid]
        delete streamingContent.value[sid]
        delete toolCalls.value[sid]
      } else {
        streamingContent.value[sid] = payload.content || ''
      }
    })

    // Listen for permission requests from Claude
    unlistenPermission = await listen('claude-permission', (event: any) => {
      const payload = event.payload
      if (!payload) return
      const sid = payload.session_id
      console.log(`[Permission] Session ${sid}:`, payload.request)
      pendingPermissions.value = {
        ...pendingPermissions.value,
        [sid]: payload.request,
      }
    })

    // Listen for trust prompts from Claude
    await listen('claude-trust-prompt', (event: any) => {
      const payload = event.payload
      if (!payload) return
      const sid = payload.session_id
      console.log(`[Trust] Session ${sid}:`, payload.message)
      pendingTrustPrompts.value = {
        ...pendingTrustPrompts.value,
        [sid]: payload,
      }
    })
  }

  function clearPermission(sessionId: string) {
    const next = { ...pendingPermissions.value }
    delete next[sessionId]
    pendingPermissions.value = next
  }

  function getPendingPermission(sessionId: string): any {
    return pendingPermissions.value[sessionId] || null
  }

  function clearTrustPrompt(sessionId: string) {
    const next = { ...pendingTrustPrompts.value }
    delete next[sessionId]
    pendingTrustPrompts.value = next
  }

  function getPendingTrustPrompt(sessionId: string): any {
    return pendingTrustPrompts.value[sessionId] || null
  }

  function markPending(sessionId: string) {
    // Replace Set to trigger Vue reactivity
    const next = new Set(pendingResponses.value)
    next.add(sessionId)
    pendingResponses.value = next
    streamingThinking.value[sessionId] = ''
    streamingContent.value[sessionId] = ''
    toolCalls.value[sessionId] = []
  }

  function clearPending(sessionId: string) {
    // Replace Set to trigger Vue reactivity
    const next = new Set(pendingResponses.value)
    next.delete(sessionId)
    pendingResponses.value = next
    delete streamingThinking.value[sessionId]
    delete streamingContent.value[sessionId]
    delete toolCalls.value[sessionId]
  }

  function isPending(sessionId: string): boolean {
    return pendingResponses.value.has(sessionId)
  }

  function getStreamingThinking(sessionId: string): string {
    return streamingThinking.value[sessionId] || ''
  }

  function getStreamingContent(sessionId: string): string {
    return streamingContent.value[sessionId] || ''
  }

  function getToolCalls(sessionId: string): ToolCall[] {
    return toolCalls.value[sessionId] || []
  }

  async function fetchSessions() {
    loading.value = true
    try {
      sessions.value = await api.listSessions()
    } finally {
      loading.value = false
    }
  }

  async function createSession(taskId: string, providerId: string) {
    const now = new Date().toISOString()
    const session = await api.createSession(
      crypto.randomUUID(),
      taskId,
      providerId,
      now,
      now
    )
    sessions.value.push(session)
    return session
  }

  async function getOrCreateActiveSession(projectId: string, providerId: string) {
    const existingSession = sessions.value.find(
      (s) => s.taskId === projectId && s.providerId === providerId && (s.status === 'created' || s.status === 'running')
    )
    if (existingSession) {
      currentSession.value = existingSession
      return existingSession
    }
    const newSession = await createSession(projectId, providerId)
    currentSession.value = newSession
    return newSession
  }

  function getActiveSession(projectId: string, providerId: string): Session | undefined {
    return sessions.value.find(
      (s) => s.taskId === projectId && s.providerId === providerId && (s.status === 'created' || s.status === 'running')
    )
  }

  async function updateSessionStatus(id: string, status: SessionStatus) {
    await api.updateSessionStatus(id, status)
    const session = sessions.value.find((s) => s.id === id)
    if (session) {
      session.status = status
      session.updatedAt = new Date().toISOString()
    }
  }

  async function deleteSession(id: string) {
    await api.deleteSession(id)
    await api.deleteMessages(id)
    sessions.value = sessions.value.filter((s) => s.id !== id)
    messages.value = messages.value.filter((m) => m.sessionId !== id)
    if (currentSession.value?.id === id) {
      currentSession.value = null
    }
  }

  async function addMessage(message: Message) {
    await api.createMessage(
      message.id,
      message.sessionId,
      message.type,
      message.content,
      message.createdAt,
      message.thinking
    )
    messages.value.push(message)
  }

  async function fetchMessages(sessionId: string) {
    const raw = await api.listMessages(sessionId)
    messages.value = raw.map((m: any) => ({
      ...m,
      thinking: m.metadata?.thinking ?? undefined,
    }))
  }

  function setCurrentSession(session: Session | null) {
    currentSession.value = session
  }

  function initSessionContext(sessionId: string, projectId: string, projectName: string, projectPath: string) {
    sessionContexts.value[sessionId] = {
      workingDirectory: projectPath,
      projectId,
      projectName,
      projectPath,
    }
  }

  function getSessionContext(sessionId: string): SessionContext | undefined {
    return sessionContexts.value[sessionId]
  }

  function updateWorkingDirectory(sessionId: string, directory: string) {
    if (sessionContexts.value[sessionId]) {
      sessionContexts.value[sessionId].workingDirectory = directory
    }
  }

  function deleteSessionContext(sessionId: string) {
    delete sessionContexts.value[sessionId]
  }

  return {
    sessions,
    currentSession,
    messages,
    loading,
    pendingResponses,
    streamingThinking,
    streamingContent,
    sessionContexts,
    toolCalls,
    fetchSessions,
    createSession,
    getOrCreateActiveSession,
    getActiveSession,
    updateSessionStatus,
    deleteSession,
    addMessage,
    fetchMessages,
    setCurrentSession,
    initClaudeListener,
    markPending,
    clearPending,
    isPending,
    getStreamingThinking,
    getStreamingContent,
    getToolCalls,
    pendingPermissions,
    clearPermission,
    getPendingPermission,
    pendingTrustPrompts,
    clearTrustPrompt,
    getPendingTrustPrompt,
    initSessionContext,
    getSessionContext,
    updateWorkingDirectory,
    deleteSessionContext,
  }
})
