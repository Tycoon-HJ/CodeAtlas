<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'
import { useAgentStore } from '@/stores/agent'
import { useSessionStore } from '@/stores/session'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ 'update:show': [value: boolean] }>()

const router = useRouter()
const projectStore = useProjectStore()
const agentStore = useAgentStore()
const sessionStore = useSessionStore()

const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

interface CommandItem {
  id: string
  label: string
  description: string
  icon: string
  action: () => void
}

const allCommands = computed<CommandItem[]>(() => {
  const commands: CommandItem[] = []

  // Projects
  for (const p of projectStore.projects) {
    commands.push({
      id: `project-${p.id}`,
      label: p.name,
      description: p.path,
      icon: 'folder',
      action: () => {
        router.push(`/project/${p.id}`)
        close()
      },
    })
  }

  // Actions
  commands.push(
    { id: 'new-chat', label: '新建对话', description: '创建新的 AI 对话', icon: 'chat', action: () => { router.push(`/project/${projectStore.currentProject?.id}/chat`); close() } },
    { id: 'agents', label: 'Agent 管理', description: '管理 AI Agent', icon: 'agent', action: () => { router.push('/agents'); close() } },
    { id: 'settings', label: '设置', description: '应用设置', icon: 'settings', action: () => { router.push('/settings'); close() } },
    { id: 'home', label: '回到首页', description: '返回项目列表', icon: 'home', action: () => { router.push('/'); close() } },
  )

  // Agents
  for (const p of agentStore.providers) {
    commands.push({
      id: `agent-${p.id}`,
      label: `切换到 ${p.name}`,
      description: p.enabled ? '已启用' : '未启用',
      icon: 'agent',
      action: () => {
        agentStore.setCurrentProvider(p)
        close()
      },
    })
  }

  return commands
})

const filteredCommands = computed(() => {
  if (!query.value) return allCommands.value
  const q = query.value.toLowerCase()
  return allCommands.value.filter(
    (c) => c.label.toLowerCase().includes(q) || c.description.toLowerCase().includes(q)
  )
})

watch(() => props.show, (val) => {
  if (val) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => inputRef.value?.focus())
  }
})

watch(query, () => {
  selectedIndex.value = 0
})

function close() {
  emit('update:show', false)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    close()
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredCommands.value.length - 1)
    return
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
    return
  }
  if (e.key === 'Enter') {
    e.preventDefault()
    const cmd = filteredCommands.value[selectedIndex.value]
    if (cmd) cmd.action()
    return
  }
}

function getIcon(icon: string): string {
  const icons: Record<string, string> = {
    folder: '📁', chat: '💬', agent: '🤖', settings: '⚙️', home: '🏠',
  }
  return icons[icon] ?? '📋'
}
</script>

<template>
  <Teleport to="body">
    <Transition name="palette">
      <div v-if="show" class="palette-overlay" @click.self="close" @keydown="handleKeydown">
        <div class="palette-container">
          <div class="palette-input-wrapper">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
              <path d="M11 11l3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <input
              ref="inputRef"
              v-model="query"
              class="palette-input"
              placeholder="搜索命令、项目、会话..."
              spellcheck="false"
            />
          </div>
          <div class="palette-list">
            <div
              v-for="(cmd, i) in filteredCommands"
              :key="cmd.id"
              class="palette-item"
              :class="{ selected: i === selectedIndex }"
              @click="cmd.action()"
              @mouseenter="selectedIndex = i"
            >
              <span class="item-icon">{{ getIcon(cmd.icon) }}</span>
              <div class="item-text">
                <span class="item-label">{{ cmd.label }}</span>
                <span class="item-desc">{{ cmd.description }}</span>
              </div>
            </div>
            <div v-if="filteredCommands.length === 0" class="palette-empty">
              没有匹配的命令
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped lang="scss">
.palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.palette-container {
  width: 520px;
  max-height: 400px;
  background: var(--bg-secondary);
  border: 0.5px solid var(--border-default);
  border-radius: 12px;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.palette-input-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 0.5px solid var(--border-default);
}

.search-icon {
  flex-shrink: 0;
  color: var(--text-tertiary);
}

.palette-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 15px;
  font-family: var(--font-sans);
  outline: none;

  &::placeholder {
    color: var(--text-tertiary);
  }
}

.palette-list {
  overflow-y: auto;
  padding: 6px;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: background var(--transition-fast);

  &.selected {
    background: var(--primary-light);
  }

  &:hover {
    background: var(--bg-hover);
  }
}

.item-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.item-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.item-label {
  font-size: 13px;
  color: var(--text-primary);
}

.item-desc {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.palette-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 13px;
}

// Transitions
.palette-enter-active {
  transition: opacity 0.15s ease;

  .palette-container {
    transition: transform 0.15s ease, opacity 0.15s ease;
  }
}

.palette-leave-active {
  transition: opacity 0.1s ease;

  .palette-container {
    transition: transform 0.1s ease, opacity 0.1s ease;
  }
}

.palette-enter-from, .palette-leave-to {
  opacity: 0;

  .palette-container {
    transform: scale(0.96) translateY(-8px);
    opacity: 0;
  }
}
</style>
