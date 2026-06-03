import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Session, Message, SessionStatus, ToolCall } from '@/types'
import * as api from '@/api'

// 会话上下文
export interface SessionContext {
  workingDirectory: string // 当前工作目录
  projectId: string // 项目 ID
  projectName: string // 项目名称
  projectPath: string // 项目路径
}

// 工具调用记录（用于流式显示）
export interface StreamingToolCall {
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
  const toolCalls = ref<Record<string, StreamingToolCall[]>>({})
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

    unlistenOutput = await listen('claude-output', (event: any) => {
      const payload = event.payload
      if (!payload) return

      const sid = payload.session_id

      if (payload.done) {
        // Replace Set to trigger Vue reactivity (Set.delete is not reactive)
        const next = new Set(pendingResponses.value)
        next.delete(sid)
        pendingResponses.value = next

        const content = payload.content || ''
        const thinking = streamingThinking.value[sid] || payload.thinking || ''
        const toolCallsData = payload.toolCalls || toolCalls.value[sid] || []

        // Check if this is an error message
        const isError = content.startsWith('无法启动') || content.startsWith('Error:') || content.startsWith('错误:') || content.includes('not found') || content.includes('failed')

        // Build metadata with thinking and tool calls
        const metadata: Record<string, any> = {}
        if (thinking) {
          metadata.thinking = thinking
        }
        if (toolCallsData && toolCallsData.length > 0) {
          metadata.toolCalls = toolCallsData
        }

        const msg: Message = {
          id: crypto.randomUUID(),
          sessionId: sid,
          type: isError ? 'system' : 'assistant',
          content: content || 'Claude CLI 未返回内容',
          thinking: thinking || undefined,
          metadata: Object.keys(metadata).length > 0 ? metadata : undefined,
          createdAt: new Date().toISOString(),
        }

        // Clear streaming state immediately (before persistence)
        delete streamingThinking.value[sid]
        delete streamingContent.value[sid]
        delete toolCalls.value[sid]

        // Push to memory immediately (UI updates right away)
        messages.value.push(msg)

        // Persist to backend in background (non-blocking)
        api.createMessage(msg.id, msg.sessionId, msg.type, msg.content, msg.createdAt, msg.thinking, msg.metadata)
          .then(() => console.log('[Session] Message persisted:', { sessionId: sid, type: msg.type, contentLength: content.length }))
          .catch((persistErr) => console.error('[Session] Failed to persist message:', persistErr))
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

  function getToolCalls(sessionId: string): StreamingToolCall[] {
    return toolCalls.value[sessionId] || []
  }

  async function fetchSessions() {
    loading.value = true
    try {
      const allSessions = await api.listSessions()
      // Filter out sessions without messages (empty sessions)
      const sessionsWithMessages: Session[] = []
      for (const session of allSessions) {
        const messages = await api.listMessages(session.id)
        if (messages.length > 0) {
          sessionsWithMessages.push(session)
        }
      }
      sessions.value = sessionsWithMessages
    } finally {
      loading.value = false
    }
  }

  async function createSession(taskId: string, providerId: string, saveToBackend: boolean = true) {
    const now = new Date().toISOString()
    const sessionId = crypto.randomUUID()

    if (saveToBackend) {
      // Save to backend immediately
      const session = await api.createSession(sessionId, taskId, providerId, now, now)
      sessions.value.push(session)
      return session
    } else {
      // Create temporary session (not saved to backend yet)
      const session: Session = {
        id: sessionId,
        taskId,
        providerId,
        status: 'created',
        tokenInput: 0,
        tokenOutput: 0,
        cost: 0,
        createdAt: now,
        updatedAt: now,
      }
      sessions.value.push(session)
      return session
    }
  }

  async function saveSessionToBackend(session: Session) {
    try {
      const now = new Date().toISOString()
      // Update the session's updatedAt timestamp
      session.updatedAt = now
      await api.createSession(
        session.id,
        session.taskId,
        session.providerId,
        session.createdAt,
        now
      )
      console.log('[Session] Session saved to backend:', session.id)
    } catch (e) {
      console.error('[Session] Failed to save session to backend:', e)
    }
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

  async function updateSessionTitle(id: string, title: string) {
    await api.updateSessionTitle(id, title)
    const session = sessions.value.find((s) => s.id === id)
    if (session) {
      session.title = title
    }
  }

  function addMessage(message: Message) {
    messages.value.push(message)
    // Persist in background (non-blocking)
    api.createMessage(
      message.id,
      message.sessionId,
      message.type,
      message.content,
      message.createdAt,
      message.thinking,
      message.metadata
    ).catch((e) => console.error('[Session] Failed to persist message:', e))
  }

  async function fetchMessages(sessionId: string) {
    const raw = await api.listMessages(sessionId)
    console.log(`[Session] fetchMessages for ${sessionId}:`, raw.length, 'messages', raw)
    messages.value = raw.map((m: any) => ({
      ...m,
      thinking: m.metadata?.thinking ?? undefined,
      metadata: m.metadata ?? undefined,
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
    saveSessionToBackend,
    getOrCreateActiveSession,
    getActiveSession,
    updateSessionStatus,
    deleteSession,
    addMessage,
    updateSessionTitle,
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
