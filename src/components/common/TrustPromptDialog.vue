<script setup lang="ts">
import { NModal, NButton, NSpace, NText } from 'naive-ui'

defineProps<{
  show: boolean
  message: string
}>()

const emit = defineEmits<{
  trust: []
  deny: []
}>()
</script>

<template>
  <NModal
    :show="show"
    :mask-closable="false"
    :close-on-esc="false"
    preset="card"
    title=""
    :style="{ width: '440px' }"
    :segmented="{ content: true, footer: true }"
  >
    <div class="trust-content">
      <div class="trust-icon">
        <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
          <circle cx="20" cy="20" r="18" stroke="var(--primary)" stroke-width="2"/>
          <path d="M14 20l4 4 8-8" stroke="var(--primary)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <div class="trust-info">
        <NText strong style="font-size: 16px; color: var(--text-primary); display: block; margin-bottom: 8px">
          工作目录信任确认
        </NText>
        <NText style="font-size: 13px; color: var(--text-secondary); line-height: 1.6">
          {{ message || 'Claude Code 需要您确认是否信任此工作目录。允许后 Claude 将可以读取、编辑和执行此目录中的文件。' }}
        </NText>
      </div>
    </div>

    <template #footer>
      <NSpace justify="end">
        <NButton @click="emit('deny')" quaternary>
          不信任，退出
        </NButton>
        <NButton type="primary" @click="emit('trust')">
          信任此目录
        </NButton>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped lang="scss">
.trust-content {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.trust-icon {
  flex-shrink: 0;
  margin-top: 4px;
}

.trust-info {
  flex: 1;
  min-width: 0;
}
</style>
