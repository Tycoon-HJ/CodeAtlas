<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import '@xterm/xterm/css/xterm.css'

const props = defineProps<{
  lines: string[]
  active?: boolean
}>()

const terminalRef = ref<HTMLDivElement | null>(null)
const terminal = ref<Terminal | null>(null)
const fitAddon = ref<FitAddon | null>(null)
const searchAddon = ref<SearchAddon | null>(null)
const searchQuery = ref('')
const showSearch = ref(false)

onMounted(() => {
  if (!terminalRef.value) return

  const term = new Terminal({
    theme: {
      background: '#1a1a2e',
      foreground: '#e0e0e0',
      cursor: '#409CFF',
      selectionBackground: 'rgba(64, 156, 255, 0.3)',
      black: '#1a1a2e',
      red: '#FF453A',
      green: '#30D158',
      yellow: '#FFD60A',
      blue: '#409CFF',
      magenta: '#BF5AF2',
      cyan: '#64D2FF',
      white: '#E0E0E0',
    },
    fontSize: 13,
    fontFamily: 'var(--font-mono)',
    lineHeight: 1.4,
    cursorBlink: false,
    disableStdin: true,
    scrollback: 5000,
    convertEol: true,
  })

  const fit = new FitAddon()
  const search = new SearchAddon()
  term.loadAddon(fit)
  term.loadAddon(search)

  term.open(terminalRef.value)
  fit.fit()

  terminal.value = term
  fitAddon.value = fit
  searchAddon.value = search

  // Write initial lines
  for (const line of props.lines) {
    term.writeln(line)
  }

  // Resize observer
  const ro = new ResizeObserver(() => fit.fit())
  ro.observe(terminalRef.value)

  onBeforeUnmount(() => {
    ro.disconnect()
    term.dispose()
  })
})

watch(() => props.lines, (newLines, oldLines) => {
  if (!terminal.value) return
  const added = newLines.slice(oldLines?.length ?? 0)
  for (const line of added) {
    terminal.value.writeln(line)
  }
}, { deep: true })

watch(() => props.active, () => {
  setTimeout(() => fitAddon.value?.fit(), 50)
})

function clearTerminal() {
  terminal.value?.clear()
}

function handleSearch() {
  if (searchQuery.value && searchAddon.value) {
    searchAddon.value.findNext(searchQuery.value)
  }
}

function handleSearchPrev() {
  if (searchQuery.value && searchAddon.value) {
    searchAddon.value.findPrevious(searchQuery.value)
  }
}
</script>

<template>
  <div class="terminal-panel">
    <div class="terminal-toolbar">
      <div class="terminal-actions">
        <button class="toolbar-btn" @click="clearTerminal" title="清屏">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M3 3l10 10M13 3L3 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="toolbar-btn" @click="showSearch = !showSearch" title="搜索">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <circle cx="7" cy="7" r="4.5" stroke="currentColor" stroke-width="1.2"/>
            <path d="M10.5 10.5L14 14" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
      <div v-if="showSearch" class="terminal-search">
        <input
          v-model="searchQuery"
          class="search-input"
          placeholder="搜索..."
          @keydown.enter="handleSearch"
          @keydown.shift.enter="handleSearchPrev"
        />
      </div>
    </div>
    <div ref="terminalRef" class="terminal-container" />
  </div>
</template>

<style scoped lang="scss">
.terminal-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a2e;
}

.terminal-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-bottom: 0.5px solid var(--border-color);
  background: var(--bg-secondary);
  flex-shrink: 0;
  gap: 8px;
}

.terminal-actions {
  display: flex;
  gap: 4px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 4px;
  cursor: pointer;

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
}

.terminal-search {
  flex: 1;
  max-width: 240px;
}

.search-input {
  width: 100%;
  padding: 3px 8px;
  border: 0.5px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  font-family: var(--font-mono);
  outline: none;

  &:focus {
    border-color: var(--accent-blue);
  }
}

.terminal-container {
  flex: 1;
  padding: 4px;
  overflow: hidden;

  :deep(.xterm) {
    padding: 4px;
  }
}
</style>
