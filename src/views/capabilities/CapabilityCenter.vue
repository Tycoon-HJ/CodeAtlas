<script setup lang="ts">
import { NCard, NTabs, NTabPane, NText, NButton, NSpace } from 'naive-ui'
</script>

<template>
  <div style="padding: 24px">
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText tag="h2" style="font-size: 20px; font-weight: 600">能力中心</NText>
    </NSpace>
    <NCard>
      <NTabs>
        <NTabPane name="mcp" tab="MCP">
          <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
            <NText>Model Context Protocol 管理</NText>
            <NButton type="primary" size="small" @click="$router.push('/workspace/capabilities/mcp')">
              查看全部
            </NButton>
          </NSpace>
        </NTabPane>
        <NTabPane name="skill" tab="Skill">
          <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
            <NText>Skill 管理</NText>
            <NButton type="primary" size="small" @click="$router.push('/workspace/capabilities/skill')">
              查看全部
            </NButton>
          </NSpace>
        </NTabPane>
        <NTabPane name="prompt" tab="Prompt">
          <NText depth="3">Prompt 模板管理</NText>
        </NTabPane>
        <NTabPane name="workflow" tab="Workflow">
          <NText depth="3">工作流管理</NText>
        </NTabPane>
      </NTabs>
    </NCard>
  </div>
</template>
