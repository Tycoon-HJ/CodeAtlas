<script setup lang="ts">
import { NCard, NList, NListItem, NSwitch, NSpace, NText, NButton, NModal, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { ref, computed, onMounted } from 'vue'
import { useCapabilityStore } from '@/stores/capability'

const capabilityStore = useCapabilityStore()
const message = useMessage()
const showCreate = ref(false)
const newName = ref('')
const newDesc = ref('')

const presetSkills = [
  { name: 'Java Skill', description: 'Java 开发辅助' },
  { name: 'SpringBoot Skill', description: 'SpringBoot 开发辅助' },
  { name: 'Vue3 Skill', description: 'Vue3 开发辅助' },
  { name: 'React Skill', description: 'React 开发辅助' },
  { name: 'Python Skill', description: 'Python 开发辅助' },
]

const allSkills = computed(() => {
  const custom = capabilityStore.skillCapabilities
  return [
    ...presetSkills.map((p) => ({
      ...p,
      id: `preset-${p.name}`,
      enabled: custom.some((c) => c.name === p.name && c.enabled),
    })),
    ...custom.filter((c) => !presetSkills.some((p) => p.name === c.name)),
  ]
})

onMounted(() => capabilityStore.fetchCapabilities())

async function toggleSkill(name: string) {
  const existing = capabilityStore.capabilities.find((c) => c.name === name && c.type === 'skill')
  if (existing) {
    await capabilityStore.toggleCapability(existing.id)
  } else {
    await capabilityStore.addCapability({
      type: 'skill',
      name,
      description: presetSkills.find((p) => p.name === name)?.description,
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
    type: 'skill',
    name: newName.value,
    description: newDesc.value || undefined,
    tags: [],
    enabled: true,
    config: {},
  })
  showCreate.value = false
  newName.value = ''
  newDesc.value = ''
  message.success('自定义 Skill 已添加')
}
</script>

<template>
  <div style="padding: 24px">
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText tag="h2" style="font-size: 20px; font-weight: 600">Skill 管理</NText>
      <NSpace>
        <NButton @click="message.info('导入功能开发中')">导入</NButton>
        <NButton type="primary" @click="showCreate = true">创建 Skill</NButton>
      </NSpace>
    </NSpace>

    <NList bordered>
      <NListItem v-for="skill in allSkills" :key="skill.name">
        <NSpace justify="space-between" align="center">
          <div>
            <NText strong>{{ skill.name }}</NText>
            <NText depth="3" style="margin-left: 8px">{{ skill.description }}</NText>
          </div>
          <NSpace>
            <NButton size="small" @click="message.info('导出功能开发中')">导出</NButton>
            <NSwitch :value="skill.enabled" @update:value="toggleSkill(skill.name)" />
          </NSpace>
        </NSpace>
      </NListItem>
    </NList>

    <NModal v-model:show="showCreate">
      <NCard title="创建 Skill" :bordered="false" style="width: 480px">
        <NForm>
          <NFormItem label="名称"><NInput v-model:value="newName" placeholder="Skill 名称" /></NFormItem>
          <NFormItem label="描述"><NInput v-model:value="newDesc" placeholder="Skill 描述" /></NFormItem>
        </NForm>
        <template #footer>
          <NSpace justify="end">
            <NButton @click="showCreate = false">取消</NButton>
            <NButton type="primary" @click="handleCreate">创建</NButton>
          </NSpace>
        </template>
      </NCard>
    </NModal>
  </div>
</template>
