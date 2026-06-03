<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useProjectStore } from '@/stores/project'

const router = useRouter()
const route = useRoute()
const projectStore = useProjectStore()

const projectId = computed(() => route.params.projectId as string | undefined)
const project = computed(() =>
  projectId.value ? projectStore.projects.find((p) => p.id === projectId.value) : null
)

const navItems = computed(() => {
  if (!project.value) return []
  return [
    { key: 'workspace', icon: 'house', label: '项目主页', path: `/project/${project.value.id}` },
    { key: 'chat', icon: 'chat', label: 'AI 对话', path: `/project/${project.value.id}/chat` },
  ]
})

function isActiveNav(key: string): boolean {
  const name = route.name as string
  if (key === 'workspace') return name === 'project-workspace'
  if (key === 'chat') return name === 'project-chat' || name === 'project-session'
  return false
}
</script>

<template>
  <div class="sidebar">
    <!-- Header -->
    <div class="sidebar-header" @click="router.push('/')">
      <span class="app-logo">CA</span>
      <span class="app-name">CodeAtlas</span>
    </div>

    <div class="sidebar-scroll">
      <!-- Current project nav -->
      <div v-if="project" class="sidebar-section">
        <div class="section-label">{{ project.name }}</div>

        <div
          v-for="item in navItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: isActiveNav(item.key) }"
          @click="router.push(item.path)"
        >
          <!-- House icon -->
          <svg v-if="item.icon === 'house'" width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 8l6-5.5L14 8M3.5 7V14h3.5v-3h3v3h3.5V7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <!-- Chat icon -->
          <svg v-if="item.icon === 'chat'" width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 3h12a1 1 0 011 1v7a1 1 0 01-1 1H5l-3 3V4a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <span>{{ item.label }}</span>
        </div>
      </div>

      <!-- No project selected -->
      <div v-if="!project" class="sidebar-section">
        <div class="nav-item" @click="router.push('/')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 8l6-5.5L14 8M3.5 7V14h3.5v-3h3v3h3.5V7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>首页</span>
        </div>
      </div>

      <!-- Global section -->
      <div class="sidebar-section global-section">
        <div class="section-label">全局</div>
        <div class="nav-item" @click="router.push('/agents')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="5" r="3" stroke="currentColor" stroke-width="1.2"/>
            <path d="M2 14c0-3.3 2.7-6 6-6s6 2.7 6 6" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <span>Agent 管理</span>
        </div>
        <div class="nav-item" @click="router.push('/settings')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="2.5" stroke="currentColor" stroke-width="1.2"/>
            <path d="M8 1v2M8 13v2M1 8h2M13 8h2M2.9 2.9l1.5 1.5M11.6 11.6l1.5 1.5M13.1 2.9l-1.5 1.5M4.4 11.6l-1.5 1.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
          <span>设置</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  color: var(--text-primary);
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px;
  cursor: pointer;
  flex-shrink: 0;
  transition: background var(--transition-fast);
  border-bottom: 1px solid var(--border-default);

  &:hover {
    background: var(--bg-hover);
  }
}

.app-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-md);
  background: var(--primary);
  color: #fff;
  font-size: var(--text-xs);
  font-weight: 700;
}

.app-name {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.sidebar-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.sidebar-section {
  padding: 8px 0;

  &.global-section {
    border-top: 1px solid var(--border-default);
    margin-top: auto;
    padding-top: 8px;
  }
}

.section-label {
  font-size: var(--text-xs);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-tertiary);
  padding: 8px 16px 4px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: var(--text-base);
  transition: all var(--transition-fast);
  border-radius: var(--radius-md);
  margin: 2px 8px;

  svg {
    flex-shrink: 0;
    width: 16px;
    height: 16px;
  }

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  &.active {
    background: var(--primary-light);
    color: var(--primary);
    font-weight: 500;
  }
}
</style>
