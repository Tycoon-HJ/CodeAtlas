<script setup lang="ts">
import { NButton, NSpace, NBreadcrumb, NBreadcrumbItem } from 'naive-ui'
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'

defineProps<{
  showMenuToggle?: boolean
}>()

const emit = defineEmits<{
  toggleMenu: []
}>()

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()

const projectId = computed(() => route.params.projectId as string | undefined)
const project = computed(() =>
  projectId.value ? projectStore.projects.find((p) => p.id === projectId.value) : null
)

const breadcrumbs = computed(() => {
  const items: { label: string; to?: string }[] = []
  items.push({ label: 'CodeAtlas', to: '/' })
  if (project.value) {
    items.push({ label: project.value.name, to: `/project/${project.value.id}` })
    const name = route.name as string
    if (name === 'project-chat') items.push({ label: 'AI 对话' })
    else if (name === 'project-session') items.push({ label: 'AI 对话' })
  }
  return items
})
</script>

<template>
  <div class="top-navbar">
    <div class="navbar-left">
      <button
        v-if="showMenuToggle"
        class="icon-btn"
        @click="emit('toggleMenu')"
        title="切换侧边栏"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
      <NBreadcrumb>
        <NBreadcrumbItem
          v-for="(item, i) in breadcrumbs"
          :key="i"
          @click="item.to && router.push(item.to)"
        >
          {{ item.label }}
        </NBreadcrumbItem>
      </NBreadcrumb>
    </div>

    <div class="navbar-right">
      <button class="icon-btn" @click="router.push('/')" title="项目列表">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M2 3h5l1 1h5a1 1 0 011 1v8a1 1 0 01-1 1H2a1 1 0 01-1-1V4a1 1 0 011-1z" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.top-navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--topbar-height);
  padding: 0 16px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-default);
  flex-shrink: 0;
}

.navbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.navbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
}

@media (max-width: 767px) {
  .navbar-left {
    gap: 8px;
  }
}
</style>
