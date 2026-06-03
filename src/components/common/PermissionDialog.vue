<script setup lang="ts">
import { NModal, NButton, NSpace, NText } from 'naive-ui'
import { computed } from 'vue'

const props = defineProps<{
  show: boolean
  request: any
}>()

const emit = defineEmits<{
  allow: []
  deny: []
}>()

const title = computed(() => {
  if (!props.request) return '权限请求'
  const req = props.request
  // Try to extract meaningful info from the permission request
  if (req.permission) return `权限请求: ${req.permission}`
  if (req.tool) return `工具权限: ${req.tool}`
  if (req.command) return `命令权限: ${req.command}`
  return '权限请求'
})

const description = computed(() => {
  if (!props.request) return ''
  const req = props.request
  // Build a human-readable description
  const parts: string[] = []
  if (req.message) parts.push(req.message)
  if (req.description) parts.push(req.description)
  if (req.command) parts.push(`命令: ${req.command}`)
  if (req.path) parts.push(`路径: ${req.path}`)
  if (req.tool) parts.push(`工具: ${req.tool}`)
  if (req.input && Object.keys(req.input).length > 0) parts.push(`参数: ${JSON.stringify(req.input, null, 2)}`)
  return parts.join('\n') || JSON.stringify(req, null, 2)
})
</script>

<template>
  <NModal
    :show="show"
    :mask-closable="false"
    :close-on-esc="false"
    preset="card"
    title=""
    :style="{ width: '480px' }"
    :segmented="{ content: true, footer: true }"
  >
    <div class="permission-content">
      <div class="permission-icon">
        <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
          <circle cx="20" cy="20" r="18" stroke="var(--warning)" stroke-width="2"/>
          <path d="M20 12v10M20 26v2" stroke="var(--warning)" stroke-width="2.5" stroke-linecap="round"/>
        </svg>
      </div>
      <div class="permission-info">
        <NText strong style="font-size: 16px; color: var(--text-primary); display: block; margin-bottom: 8px">
          {{ title }}
        </NText>
        <pre class="permission-detail">{{ description }}</pre>
      </div>
    </div>

    <template #footer>
      <NSpace justify="end">
        <NButton @click="emit('deny')" quaternary>
          拒绝
        </NButton>
        <NButton type="warning" @click="emit('allow')">
          允许
        </NButton>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped lang="scss">
.permission-content {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.permission-icon {
  flex-shrink: 0;
  margin-top: 4px;
}

.permission-info {
  flex: 1;
  min-width: 0;
}

.permission-detail {
  font-size: 13px;
  color: var(--text-secondary);
  background: var(--bg-primary);
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-default);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: var(--font-mono);
  margin: 0;
  max-height: 200px;
  overflow-y: auto;
}
</style>
