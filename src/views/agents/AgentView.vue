<script setup lang="ts">
import { NCard, NList, NListItem, NSwitch, NSpace, NText, NTag } from 'naive-ui'
import { useAgentStore } from '@/stores/agent'

const agentStore = useAgentStore()
</script>

<template>
  <div class="agent-view">
    <h2 class="agent-title">Agent 管理</h2>
    <div class="agent-list">
      <div v-for="provider in agentStore.providers" :key="provider.id" class="agent-item">
        <div class="agent-info">
          <div class="agent-icon">
            <svg width="18" height="18" viewBox="0 0 16 16" fill="none">
              <circle cx="8" cy="5" r="3" stroke="currentColor" stroke-width="1.2"/>
              <path d="M2 14c0-3.3 2.7-6 6-6s6 2.7 6 6" stroke="currentColor" stroke-width="1.2"/>
            </svg>
          </div>
          <div class="agent-details">
            <span class="agent-name">{{ provider.name }}</span>
            <span class="agent-status" :class="{ enabled: provider.enabled }">
              {{ provider.enabled ? '已启用' : '未启用' }}
            </span>
          </div>
        </div>
        <NSwitch :value="provider.enabled" @update:value="agentStore.toggleProvider(provider.id)" />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.agent-view {
  padding: 32px;
  max-width: 720px;
  margin: 0 auto;
  overflow-y: auto;
  height: 100%;
}

.agent-title {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 24px 0;
  color: var(--text-primary);
  font-family: var(--font-sans);
}

.agent-list {
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  overflow: hidden;
}

.agent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-default);
  transition: background var(--transition-fast);

  &:last-child {
    border-bottom: none;
  }

  &:hover {
    background: var(--bg-hover);
  }
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.agent-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-light);
  border-radius: var(--radius-lg);
  color: var(--primary);
}

.agent-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.agent-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.agent-status {
  font-size: 12px;
  color: var(--text-tertiary);

  &.enabled {
    color: var(--success);
  }
}
</style>
