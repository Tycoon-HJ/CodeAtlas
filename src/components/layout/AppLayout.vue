<script setup lang="ts">
import { NLayout, NLayoutSider, NLayoutContent } from 'naive-ui'
import { onMounted, ref, onUnmounted } from 'vue'
import TopNavbar from './TopNavbar.vue'
import LeftSidebar from './LeftSidebar.vue'
import StatusBar from './StatusBar.vue'
import CommandPalette from '@/components/common/CommandPalette.vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSessionStore } from '@/stores/session'
import { useAgentStore } from '@/stores/agent'
import { useSettingsStore } from '@/stores/settings'

const workspaceStore = useWorkspaceStore()
const sessionStore = useSessionStore()
const agentStore = useAgentStore()
const settingsStore = useSettingsStore()
const ready = ref(false)
const siderCollapsed = ref(false)
const windowWidth = ref(window.innerWidth)
const showCommandPalette = ref(false)
const terminalPanelVisible = ref(false)
const terminalPanelHeight = ref(200)

onMounted(async () => {
  await settingsStore.loadSettings()
  await workspaceStore.init()
  await sessionStore.initClaudeListener()
  await agentStore.syncProviderAvailability()
  ready.value = true

  window.addEventListener('resize', () => {
    windowWidth.value = window.innerWidth
    if (window.innerWidth < 768) {
      siderCollapsed.value = true
    }
  })
  if (window.innerWidth < 768) {
    siderCollapsed.value = true
  }

  // Global keyboard shortcuts
  window.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})

function handleGlobalKeydown(e: KeyboardEvent) {
  // Cmd+K: Command Palette
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    showCommandPalette.value = !showCommandPalette.value
  }
  // Cmd+`: Toggle terminal
  if ((e.metaKey || e.ctrlKey) && e.key === '`') {
    e.preventDefault()
    terminalPanelVisible.value = !terminalPanelVisible.value
  }
}

function toggleSider() {
  siderCollapsed.value = !siderCollapsed.value
}

function toggleTerminal() {
  terminalPanelVisible.value = !terminalPanelVisible.value
}

// Terminal resize
const isResizing = ref(false)
function startResize(e: MouseEvent) {
  isResizing.value = true
  const startY = e.clientY
  const startHeight = terminalPanelHeight.value

  const onMove = (ev: MouseEvent) => {
    const delta = startY - ev.clientY
    terminalPanelHeight.value = Math.max(100, Math.min(500, startHeight + delta))
  }
  const onUp = () => {
    isResizing.value = false
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
}
</script>

<template>
  <div v-if="!ready" class="loading-screen">
    <div class="loading-spinner"></div>
  </div>
  <div v-else class="app-shell">
    <!-- macOS window chrome -->
    <div class="window-chrome">
      <div class="traffic-lights">
        <span class="light red"></span>
        <span class="light yellow"></span>
        <span class="light green"></span>
      </div>
    </div>

    <div class="app-body">
      <!-- Sidebar -->
      <div class="sidebar-container" :class="{ collapsed: siderCollapsed }">
        <LeftSidebar />
      </div>

      <!-- Main area -->
      <div class="main-container">
        <!-- Top navbar -->
        <TopNavbar
          :show-menu-toggle="windowWidth < 768"
          :terminal-visible="terminalPanelVisible"
          @toggle-menu="toggleSider"
          @toggle-terminal="toggleTerminal"
        />

        <!-- Content -->
        <div class="content-wrapper">
          <div class="content-scroll" :style="{ flex: terminalPanelVisible ? '1 1 0' : '1 1 0' }">
            <RouterView />
          </div>

          <!-- Terminal panel -->
          <div v-if="terminalPanelVisible" class="terminal-panel" :style="{ height: terminalPanelHeight + 'px' }">
            <div class="terminal-resize-handle" @mousedown="startResize"></div>
            <StatusBar />
          </div>
        </div>
      </div>
    </div>

    <!-- Command Palette -->
    <CommandPalette v-model:show="showCommandPalette" />
  </div>
</template>

<style scoped lang="scss">
.loading-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100vw;
  background: var(--bg-primary);
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-blue);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.app-shell {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  border-radius: 10px;
  overflow: hidden;
}

// ─── Window Chrome (traffic lights) ───
.window-chrome {
  height: 38px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  background: var(--bg-secondary);
  border-bottom: 0.5px solid var(--border-color);
  -webkit-app-region: drag;
  flex-shrink: 0;
}

.traffic-lights {
  display: flex;
  gap: 8px;
  -webkit-app-region: no-drag;

  .light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    cursor: pointer;
    transition: opacity var(--transition-fast);

    &:hover {
      opacity: 0.8;
    }

    &.red { background: var(--traffic-red); }
    &.yellow { background: var(--traffic-yellow); }
    &.green { background: var(--traffic-green); }
  }
}

// ─── Body layout ───
.app-body {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.sidebar-container {
  width: var(--sidebar-width);
  flex-shrink: 0;
  background: var(--bg-primary);
  border-right: 0.5px solid var(--border-color);
  transition: width var(--transition-normal), opacity var(--transition-normal);
  overflow: hidden;

  &.collapsed {
    width: 0;
    opacity: 0;
  }
}

.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--bg-primary);
}

.content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.content-scroll {
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

// ─── Terminal panel ───
.terminal-panel {
  border-top: 0.5px solid var(--border-color);
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  position: relative;
}

.terminal-resize-handle {
  position: absolute;
  top: -3px;
  left: 0;
  right: 0;
  height: 6px;
  cursor: ns-resize;
  z-index: 10;

  &:hover {
    background: rgba(10, 132, 255, 0.3);
  }
}

@media (max-width: 767px) {
  .sidebar-container {
    position: absolute;
    z-index: 100;
    height: 100%;
  }
}
</style>
