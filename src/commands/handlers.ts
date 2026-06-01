import { commandRegistry, type CommandDef, type CommandResult, type CommandContext } from './registry'
import * as api from '@/api'

// ─── Local handlers ───

function helpHandler(_args: string[], _ctx: CommandContext): CommandResult {
  const commands = commandRegistry.getAll()
  const rows = commands
    .map((c) => `| ${c.icon} ${c.name} | ${c.description} | \`${c.usage}\` |`)
    .join('\n')
  return {
    type: 'text',
    content: `## 可用命令\n\n| 命令 | 说明 | 用法 |\n|------|------|------|\n${rows}\n\n> 输入 / 后可用上下箭头选择，回车执行。`,
  }
}

async function clearHandler(_args: string[], ctx: CommandContext): Promise<CommandResult> {
  await api.deleteMessages(ctx.sessionId)
  return { type: 'silent', content: '对话历史已清空' }
}

function costHandler(_args: string[], ctx: CommandContext): CommandResult {
  return {
    type: 'text',
    content: `**会话信息**\n- 会话 ID: \`${ctx.sessionId.slice(0, 8)}\`\n- 消息数: ${ctx.messageCount}\n- 工作目录: \`${ctx.workingDir}\``,
  }
}

function statusHandler(_args: string[], ctx: CommandContext): CommandResult {
  return {
    type: 'text',
    content: `**会话状态**\n- 会话 ID: \`${ctx.sessionId}\`\n- 项目: ${ctx.projectName}\n- 消息数: ${ctx.messageCount}\n- 工作目录: \`${ctx.workingDir}\``,
  }
}

function doctorHandler(_args: string[], ctx: CommandContext): CommandResult {
  const claudePath = ctx.claudePath || 'claude (默认)'
  const configPath = ctx.claudeConfigPath || '未配置'
  return {
    type: 'text',
    content: `**环境诊断**\n- Claude 路径: \`${claudePath}\`\n- 配置文件: \`${configPath}\`\n- 项目目录: \`${ctx.workingDir}\`\n\n> 如果命令不可用，请检查 Claude CLI 是否正确安装。`,
  }
}

function mcpHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return {
    type: 'text',
    content: `**MCP 服务器**\n\nMCP (Model Context Protocol) 服务器管理需要在 Claude CLI 交互模式中使用。\n\n> 在终端中运行 \`claude\` 进入交互模式，然后使用 \`/mcp\` 管理服务器。`,
  }
}

// ─── Claude prompt handlers ───
// These translate slash commands into prompts that Claude can process via -p mode

function claudePrompt(prompt: string): CommandResult {
  return { type: 'claude', content: '', prompt }
}

function compactHandler(args: string[], _ctx: CommandContext): CommandResult {
  const reason = args.length > 0 ? args.join(' ') : ''
  const prompt = reason
    ? `请压缩当前对话上下文，原因: ${reason}`
    : '请压缩当前对话上下文以节省 token，保留关键信息。'
  return claudePrompt(prompt)
}

function modelHandler(args: string[], _ctx: CommandContext): CommandResult {
  if (args.length > 0) {
    return claudePrompt(`请切换到模型: ${args[0]}`)
  }
  return claudePrompt('请显示当前使用的模型信息，包括模型名称、版本和能力。')
}

function memoryHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('请管理你的持久记忆。显示当前记忆内容，如果需要可以添加、修改或删除记忆条目。')
}

function permissionsHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('请显示当前的权限设置，包括允许和拒绝的操作列表。')
}

function initHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('请初始化项目的 CLAUDE.md 配置文件，包含项目结构、技术栈、编码规范等信息。')
}

function reviewHandler(args: string[], _ctx: CommandContext): CommandResult {
  const target = args.length > 0 ? args.join(' ') : '最近的代码变更'
  return claudePrompt(`请审查${target}，指出潜在问题和改进建议。`)
}

function loginHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('/login')
}

function logoutHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('/logout')
}

function terminalSetupHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('请配置终端集成，包括 shell 集成和快捷键设置。')
}

function vimHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('请切换 Vim 编辑模式。')
}

function prCommentsHandler(_args: string[], _ctx: CommandContext): CommandResult {
  return claudePrompt('请查看当前分支的 PR 评论和反馈。')
}

// ─── Register all commands ───

const commands: CommandDef[] = [
  { name: '/help', description: '显示帮助信息', usage: '/help', icon: '❓', handler: helpHandler },
  { name: '/compact', description: '压缩对话上下文以节省 token', usage: '/compact [说明]', icon: '🗜️', handler: compactHandler },
  { name: '/clear', description: '清空当前对话历史', usage: '/clear', icon: '🗑️', handler: clearHandler },
  { name: '/cost', description: '显示当前会话信息', usage: '/cost', icon: '💰', handler: costHandler },
  { name: '/model', description: '切换或查看当前 AI 模型', usage: '/model [模型名]', icon: '🤖', handler: modelHandler },
  { name: '/config', description: '查看或修改配置', usage: '/config', icon: '⚙️', handler: (_a, _c) => claudePrompt('请显示当前配置信息。') },
  { name: '/memory', description: '管理持久记忆', usage: '/memory', icon: '🧠', handler: memoryHandler },
  { name: '/permissions', description: '查看或修改权限设置', usage: '/permissions', icon: '🔐', handler: permissionsHandler },
  { name: '/status', description: '显示当前会话状态', usage: '/status', icon: '📊', handler: statusHandler },
  { name: '/doctor', description: '诊断 Claude CLI 环境问题', usage: '/doctor', icon: '🩺', handler: doctorHandler },
  { name: '/login', description: '登录 Anthropic 账号', usage: '/login', icon: '🔑', handler: loginHandler },
  { name: '/logout', description: '退出登录', usage: '/logout', icon: '🚪', handler: logoutHandler },
  { name: '/terminal-setup', description: '配置终端集成', usage: '/terminal-setup', icon: '💻', handler: terminalSetupHandler },
  { name: '/vim', description: '切换 Vim 编辑模式', usage: '/vim', icon: '📝', handler: vimHandler },
  { name: '/mcp', description: '管理 MCP 服务器', usage: '/mcp', icon: '🔌', handler: mcpHandler },
  { name: '/init', description: '初始化项目配置文件', usage: '/init', icon: '📋', handler: initHandler },
  { name: '/review', description: '代码审查', usage: '/review [目标]', icon: '🔍', handler: reviewHandler },
  { name: '/pr-comments', description: '查看 PR 评论', usage: '/pr-comments', icon: '💬', handler: prCommentsHandler },
]

commands.forEach((cmd) => commandRegistry.register(cmd))
