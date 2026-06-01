// Slash Command Registry — core types and registry

export interface CommandContext {
  sessionId: string
  workingDir: string
  claudePath?: string
  claudeConfigPath?: string
  messageCount: number
  projectName: string
}

export interface CommandResult {
  /** text: render as markdown, error: red bubble, silent: no output, claude: send prompt to Claude */
  type: 'text' | 'error' | 'silent' | 'claude'
  content: string
  /** For type='claude', the prompt to send */
  prompt?: string
}

export type CommandHandler = (args: string[], ctx: CommandContext) => Promise<CommandResult> | CommandResult

export interface CommandDef {
  name: string
  description: string
  usage: string
  icon: string
  handler: CommandHandler
}

export interface CommandMeta {
  name: string
  description: string
  usage: string
  icon: string
}

export function parseCommand(input: string): { name: string; args: string[] } | null {
  const trimmed = input.trim()
  if (!trimmed.startsWith('/')) return null

  const parts = trimmed.split(/\s+/)
  const name = parts[0].toLowerCase()
  const args = parts.slice(1)
  return { name, args }
}

export class SlashCommandRegistry {
  private commands = new Map<string, CommandDef>()

  register(def: CommandDef): void {
    this.commands.set(def.name, def)
  }

  get(name: string): CommandDef | undefined {
    return this.commands.get(name.toLowerCase())
  }

  getAll(): CommandDef[] {
    return Array.from(this.commands.values())
  }

  getMetaList(): CommandMeta[] {
    return this.getAll().map(({ name, description, usage, icon }) => ({
      name, description, usage, icon,
    }))
  }

  search(query: string): CommandDef[] {
    if (!query) return this.getAll()
    const q = query.toLowerCase()
    return this.getAll().filter(
      (cmd) =>
        cmd.name.toLowerCase().includes(q) ||
        cmd.description.toLowerCase().includes(q)
    )
  }

  async execute(input: string, ctx: CommandContext): Promise<CommandResult | null> {
    const parsed = parseCommand(input)
    if (!parsed) return null

    const def = this.commands.get(parsed.name)
    if (!def) {
      return { type: 'error', content: `未知命令: ${parsed.name}` }
    }

    return def.handler(parsed.args, ctx)
  }
}

export const commandRegistry = new SlashCommandRegistry()
