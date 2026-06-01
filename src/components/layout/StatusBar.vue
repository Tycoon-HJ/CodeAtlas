<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NScrollbar } from 'naive-ui'
import { useAgentStore } from '@/stores/agent'
import * as api from '@/api'

const agentStore = useAgentStore()
const activeTab = ref<'logs' | 'output'>('logs')
const logLines = ref<string[]>([])
const loading = ref(false)

async function loadLogs() {
  loading.value = true
  try {
    logLines.value = await api.getRecentLogs(200)
  } catch (e) {
    logLines.value = ['Failed to load logs: ' + e]
  }
  loading.value = false
}

onMounted(loadLogs)

function formatLogLine(line: string): { level: string; text: string } {
  if (line.includes('[ERROR]')) return { level: 'error', text: line }
  if (line.includes('[WARN]')) return { level: 'warn', text: line }
  if (line.includes('[DEBUG]')) return { level: 'debug', text: line }
  return { level: 'info', text: line }
}
</script>

<template>
  <div class="status-bar">
    <!-- Tabs -->
    <div class="terminal-tabs">
      <button
        class="terminal-tab"
        :class="{ active: activeTab === 'logs' }"
        @click="activeTab = 'logs'"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h12M2 8h8M2 12h10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
        运行日志
      </button>
      <button
        class="terminal-tab"
        :class="{ active: activeTab === 'output' }"
        @click="activeTab = 'output'"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M2 4l4 4-4 4M8 12h6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        输出
      </button>
      <div class="tab-spacer"></div>
      <span class="terminal-status">
        Agent: {{ agentStore.currentProvider?.name ?? '未选择' }}
      </span>
      <button class="terminal-action" @click="loadLogs" title="刷新日志">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M2 8a6 6 0 0111.5-2.3M14 2v4h-4M14 8a6 6 0 01-11.5 2.3M2 14v-4h4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <!-- Content -->
    <div class="terminal-content">
      <NScrollbar v-if="activeTab === 'logs'" class="terminal-scroll">
        <div v-if="loading" class="terminal-loading">加载中...</div>
        <div v-else-if="logLines.length === 0" class="terminal-empty">暂无日志</div>
        <div
          v-else
          v-for="(line, i) in logLines"
          :key="i"
          class="log-line"
          :class="formatLogLine(line).level"
        >{{ formatLogLine(line).text }}</div>
      </NScrollbar>
      <div v-else class="terminal-empty">
        输出将在代码执行时显示
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.status-bar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

// ─── Tabs ───
.terminal-tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 12px;
  border-bottom: 0.5px solid var(--border-color);
  flex-shrink: 0;
  background: var(--bg-secondary);
}

.terminal-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 500;
  border-radius: 5px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  &.active {
    color: var(--text-primary);
    background: rgba(10, 132, 255, 0.12);
  }
}

.tab-spacer {
  flex: 1;
}

.terminal-status {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-right: 8px;
}

.terminal-action {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: 4px;
  cursor: pointer;
  transition: all var(--transition-fast);

  &:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
}

// ─── Content ───
.terminal-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.terminal-scroll {
  height: 100%;
}

.terminal-loading,
.terminal-empty {
  padding: 16px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 12px;
}

.log-line {
  padding: 2px 14px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.5;

  &:hover {
    background: var(--bg-hover);
  }

  &.error {
    color: var(--accent-red);
    background: rgba(255, 69, 58, 0.05);
  }

  &.warn {
    color: var(--accent-orange);
    background: rgba(255, 159, 10, 0.05);
  }

  &.debug {
    color: var(--text-tertiary);
  }
}
</style>
