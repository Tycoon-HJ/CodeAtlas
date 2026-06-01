// Workspace
export interface Workspace {
  id: string
  name: string
  description?: string
  createdAt: string
  updatedAt: string
}

// Project
export interface Project {
  id: string
  workspaceId: string
  name: string
  path: string
  description?: string
  branch?: string
  lastOpenTime?: string
  isFavorite?: boolean
  createdAt: string
  updatedAt: string
}

// Session
export type SessionStatus = 'created' | 'running' | 'stopped' | 'failed' | 'archived'

export interface Session {
  id: string
  taskId: string
  providerId: string
  status: SessionStatus
  title?: string
  tokenInput: number
  tokenOutput: number
  cost: number
  createdAt: string
  updatedAt: string
}

// Message
export type MessageType = 'user' | 'assistant' | 'system' | 'tool'

export interface Message {
  id: string
  sessionId: string
  type: MessageType
  content: string
  thinking?: string
  metadata?: Record<string, unknown>
  createdAt: string
}

// Slash Command
export interface SlashCommand {
  name: string
  description: string
  usage: string
  examples: string[]
  icon?: string
}

// Provider
export type ProviderType = 'claude' | 'codex' | 'gemini' | 'qwen' | 'opencode'

export interface ProviderCapabilities {
  supportsCli: boolean
  supportsMcp: boolean
  supportsResume: boolean
  supportsSlashCommand: boolean
  supportsVision: boolean
  supportsAgent: boolean
}

export interface Provider {
  id: string
  type: ProviderType
  name: string
  version?: string
  enabled: boolean
  config: Record<string, unknown>
  capabilities: ProviderCapabilities
}

// Capability
export type CapabilityType = 'mcp' | 'skill' | 'prompt' | 'workflow' | 'tool' | 'agent'

export interface Capability {
  id: string
  type: CapabilityType
  name: string
  description?: string
  version?: string
  author?: string
  tags: string[]
  enabled: boolean
  config: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

// Token Stats
export interface TokenStats {
  providerId: string
  projectId?: string
  sessionId?: string
  inputTokens: number
  outputTokens: number
  duration: number
  cost: number
  date: string
}
