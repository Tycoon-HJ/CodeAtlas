<script setup lang="ts">
import { NLayout, NLayoutSider, NLayoutContent } from 'naive-ui'
import { onMounted, ref, onUnmounted } from 'vue'
import TopNavbar from './TopNavbar.vue'
import LeftSidebar from './LeftSidebar.vue'
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

onMounted(async () => {
  await settingsStore.loadSettings()
  await workspaceStore.init()
  await sessionStore.initClaudeListener()
  await agentStore.loadAgentState() // Load saved agent state first
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

  // Disable right-click context menu
  window.addEventListener('contextmenu', preventContextMenu)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
  window.removeEventListener('contextmenu', preventContextMenu)
})

function handleGlobalKeydown(e: KeyboardEvent) {
  // Cmd+K: Command Palette
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    showCommandPalette.value = !showCommandPalette.value
  }
}

function preventContextMenu(e: Event) {
  e.preventDefault()
}

function toggleSider() {
  siderCollapsed.value = !siderCollapsed.value
}
</script>

<template>
  <div v-if="!ready" class="loading-screen">
    <div class="loading-spinner"></div>
  </div>
  <div v-else class="app-shell">
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
          @toggle-menu="toggleSider"
        />

        <!-- Content -->
        <div class="content-wrapper">
          <div class="content-scroll">
            <RouterView />
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
  border: 2px solid var(--border-default);
  border-top-color: var(--primary);
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
  overflow: hidden;
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
  background: var(--bg-surface);
  border-right: 1px solid var(--border-default);
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

@media (max-width: 767px) {
  .sidebar-container {
    position: absolute;
    z-index: 100;
    height: 100%;
  }
}
</style>
