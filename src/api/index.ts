import { invoke } from '@tauri-apps/api/core'
import type { Workspace, Project, Session, Message, MessageMetadata } from '@/types'

// ─── Greet ───

export async function greet(name: string): Promise<{ message: string }> {
  return invoke('greet', { name })
}

// ─── Workspace ───

export async function listWorkspaces(): Promise<Workspace[]> {
  return invoke('list_workspaces')
}

export async function createWorkspace(
  id: string, name: string, description: string | undefined,
  createdAt: string, updatedAt: string
): Promise<Workspace> {
  return invoke('create_workspace', { id, name, description, createdAt, updatedAt })
}

export async function deleteWorkspace(id: string): Promise<boolean> {
  return invoke('delete_workspace', { id })
}

// ─── Project ───

export async function listProjects(): Promise<Project[]> {
  return invoke('list_projects')
}

export async function createProject(
  id: string, workspaceId: string, name: string, path: string,
  description: string | undefined, createdAt: string, updatedAt: string
): Promise<Project> {
  return invoke('create_project', { id, workspaceId, name, path, description, createdAt, updatedAt })
}

export async function deleteProject(id: string): Promise<boolean> {
  return invoke('delete_project', { id })
}

export async function updateProject(
  id: string, updates: { name?: string; description?: string; branch?: string; isFavorite?: boolean }
): Promise<boolean> {
  return invoke('update_project', { id, ...updates })
}

export async function openProject(id: string): Promise<boolean> {
  return invoke('open_project', { id })
}

// ─── Session ───

export async function listSessions(): Promise<Session[]> {
  return invoke('list_sessions')
}

export async function createSession(
  id: string, taskId: string, providerId: string,
  createdAt: string, updatedAt: string
): Promise<Session> {
  return invoke('create_session', { id, taskId, providerId, createdAt, updatedAt })
}

export async function updateSessionStatus(id: string, status: string): Promise<boolean> {
  return invoke('update_session_status', { id, status })
}

export async function deleteSession(id: string): Promise<boolean> {
  return invoke('delete_session', { id })
}

export async function updateSessionTitle(id: string, title: string): Promise<boolean> {
  return invoke('update_session_title', { id, title })
}

// ─── Message ───

export async function listMessages(sessionId: string): Promise<Message[]> {
  return invoke('list_messages', { sessionId })
}

export async function createMessage(
  id: string, sessionId: string, msgType: string,
  content: string, createdAt: string, thinking?: string,
  metadata?: MessageMetadata
): Promise<Message> {
  return invoke('create_message', { id, sessionId, msgType, content, createdAt, thinking, metadata })
}

export async function deleteMessages(sessionId: string): Promise<boolean> {
  return invoke('delete_messages', { sessionId })
}

// ─── Claude CLI / Provider ───

export async function sendToClaude(
  sessionId: string, message: string, workingDir: string,
  claudePath?: string, claudeConfigPath?: string
): Promise<void> {
  return invoke('send_to_claude', {
    sessionId, message, workingDir,
    claudePath: claudePath || null,
    claudeConfigPath: claudeConfigPath || null,
  })
}

export async function sendMessage(
  providerId: string, sessionId: string, message: string, workingDir: string,
  providerPath?: string, providerConfigPath?: string
): Promise<void> {
  return invoke('send_message', {
    providerId, sessionId, message, workingDir,
    providerPath: providerPath || null,
    providerConfigPath: providerConfigPath || null,
  })
}

export async function listProviders(): Promise<{ id: string; name: string; available: boolean }[]> {
  return invoke('list_providers')
}

export async function abortClaude(sessionId: string): Promise<void> {
  return invoke('abort_claude', { sessionId })
}

export async function respondPermission(sessionId: string, allow: boolean): Promise<void> {
  return invoke('respond_permission', { sessionId, allow })
}

export async function respondTrustPrompt(sessionId: string, trust: boolean): Promise<void> {
  return invoke('respond_trust_prompt', { sessionId, trust })
}

// ─── Logging ───

export async function getLogPath(): Promise<string> {
  return invoke('get_log_path')
}

export async function getRecentLogs(count: number): Promise<string[]> {
  return invoke('get_recent_logs', { count })
}

// ─── Interactive Terminal ───

export async function spawnTerminal(
  sessionId: string, workingDir: string,
  claudePath?: string, cols = 120, rows = 40
): Promise<void> {
  return invoke('spawn_terminal', {
    sessionId, workingDir,
    claudePath: claudePath || null,
    cols, rows,
  })
}

export async function writeTerminal(sessionId: string, data: string): Promise<void> {
  return invoke('write_terminal', { sessionId, data })
}

export async function resizeTerminal(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke('resize_terminal', { sessionId, cols, rows })
}

export async function closeTerminal(sessionId: string): Promise<void> {
  return invoke('close_terminal', { sessionId })
}

// ─── Settings ───

export async function loadSettings(): Promise<any> {
  return invoke('load_settings')
}

export async function saveSettings(settings: any): Promise<void> {
  return invoke('save_settings', { settings })
}

// ─── Agent State ───

export async function loadAgentState(): Promise<{ enabledProviders: string[] }> {
  return invoke('load_agent_state')
}

export async function saveAgentState(state: { enabledProviders: string[] }): Promise<void> {
  return invoke('save_agent_state', { state })
}
