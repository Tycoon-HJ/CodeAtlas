<script setup lang="ts">
import { NCard, NList, NListItem, NSwitch, NSpace, NText, NButton, NModal, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { ref, computed, onMounted } from 'vue'
import { useCapabilityStore } from '@/stores/capability'
import type { Capability } from '@/types'

const capabilityStore = useCapabilityStore()
const message = useMessage()
const showCreate = ref(false)
const newName = ref('')
const newDesc = ref('')

const presetMcps = [
  { name: 'Filesystem', description: '文件系统访问', type: 'mcp' },
  { name: 'GitHub', description: 'GitHub 集成', type: 'mcp' },
  { name: 'GitLab', description: 'GitLab 集成', type: 'mcp' },
  { name: 'MySQL', description: 'MySQL 数据库', type: 'mcp' },
  { name: 'PostgreSQL', description: 'PostgreSQL 数据库', type: 'mcp' },
  { name: 'Redis', description: 'Redis 缓存', type: 'mcp' },
  { name: 'Docker', description: 'Docker 容器', type: 'mcp' },
  { name: 'Browser', description: '浏览器控制', type: 'mcp' },
]

const allMcps = computed(() => {
  const custom = capabilityStore.mcpCapabilities
  return [
    ...presetMcps.map((p) => ({
      ...p,
      id: `preset-${p.name}`,
      enabled: custom.some((c) => c.name === p.name && c.enabled),
    })),
    ...custom.filter((c) => !presetMcps.some((p) => p.name === c.name)),
  ]
})

onMounted(() => capabilityStore.fetchCapabilities())

async function toggleMcp(name: string) {
  const existing = capabilityStore.capabilities.find((c) => c.name === name && c.type === 'mcp')
  if (existing) {
    await capabilityStore.toggleCapability(existing.id)
  } else {
    await capabilityStore.addCapability({
      type: 'mcp',
      name,
      description: presetMcps.find((p) => p.name === name)?.description,
      tags: [],
      enabled: true,
      config: {},
    })
  }
  message.success(`${name} 已切换状态`)
}

async function handleCreate() {
  if (!newName.value.trim()) { message.warning('请输入名称'); return }
  await capabilityStore.addCapability({
    type: 'mcp',
    name: newName.value,
    description: newDesc.value || undefined,
    tags: [],
    enabled: true,
    config: {},
  })
  showCreate.value = false
  newName.value = ''
  newDesc.value = ''
  message.success('自定义 MCP 已添加')
}
</script>

<template>
  <div style="padding: 24px">
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText tag="h2" style="font-size: 20px; font-weight: 600">MCP 管理</NText>
      <NButton type="primary" @click="showCreate = true">添加自定义 MCP</NButton>
    </NSpace>

    <NList bordered>
      <NListItem v-for="mcp in allMcps" :key="mcp.name">
        <NSpace justify="space-between" align="center">
          <div>
            <NText strong>{{ mcp.name }}</NText>
            <NText depth="3" style="margin-left: 8px">{{ mcp.description }}</NText>
          </div>
          <NSwitch :value="mcp.enabled" @update:value="toggleMcp(mcp.name)" />
        </NSpace>
      </NListItem>
    </NList>

    <NModal v-model:show="showCreate">
      <NCard title="添加自定义 MCP" :bordered="false" style="width: 480px">
        <NForm>
          <NFormItem label="名称"><NInput v-model:value="newName" placeholder="MCP 名称" /></NFormItem>
          <NFormItem label="描述"><NInput v-model:value="newDesc" placeholder="MCP 描述" /></NFormItem>
        </NForm>
        <template #footer>
          <NSpace justify="end">
            <NButton @click="showCreate = false">取消</NButton>
            <NButton type="primary" @click="handleCreate">添加</NButton>
          </NSpace>
        </template>
      </NCard>
    </NModal>
  </div>
</template>
