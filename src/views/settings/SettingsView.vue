<script setup lang="ts">
import { NCard, NForm, NFormItem, NInput, NSelect, NSpace, NText, NTabs, NTabPane } from 'naive-ui'
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()

const themeOptions = [
  { label: '跟随系统', value: 'system' },
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
]

function updateField(key: string, value: string) {
  settingsStore.updateSettings({ [key]: value })
}
</script>

<template>
  <div class="settings-view">
    <h2 class="settings-title">设置</h2>
    <NTabs type="line">
      <NTabPane name="general" tab="通用">
        <NCard :bordered="false" style="background: var(--bg-secondary)">
          <NForm label-placement="left" label-width="100">
            <NFormItem label="主题">
              <NSelect :value="settingsStore.settings.theme" :options="themeOptions"
                @update:value="(v: string) => settingsStore.setTheme(v as any)" />
            </NFormItem>
            <NFormItem label="语言">
              <NSelect :value="settingsStore.settings.locale"
                :options="[{ label: '简体中文', value: 'zh-CN' }, { label: 'English', value: 'en' }]"
                @update:value="(v: string) => updateField('locale', v)" />
            </NFormItem>
          </NForm>
        </NCard>
      </NTabPane>

      <NTabPane name="claude" tab="Claude">
        <NCard :bordered="false" style="background: var(--bg-secondary)">
          <NForm label-placement="left" label-width="120">
            <NFormItem label="Claude CLI 路径">
              <NInput
                :value="settingsStore.settings.claudePath"
                placeholder="留空使用默认路径 (如: /usr/local/bin/claude)"
                @update:value="(v: string) => updateField('claudePath', v)"
              />
            </NFormItem>
            <NFormItem label="配置文件路径">
              <NInput
                :value="settingsStore.settings.claudeConfigPath"
                placeholder="留空使用默认配置 (如: ~/.claude/config.json)"
                @update:value="(v: string) => updateField('claudeConfigPath', v)"
              />
            </NFormItem>
          </NForm>
          <div class="settings-hint">
            <NText depth="3" style="font-size: 12px">
              CLI 路径：指定 claude 可执行文件的完整路径。留空则自动检测。<br/>
              配置文件：指定 Claude 的配置文件路径，可用于自定义 API Key 等设置。
            </NText>
          </div>
        </NCard>
      </NTabPane>

      <NTabPane name="codex" tab="Codex">
        <NCard :bordered="false" style="background: var(--bg-secondary)">
          <NForm label-placement="left" label-width="120">
            <NFormItem label="Codex CLI 路径">
              <NInput
                :value="settingsStore.settings.codexPath"
                placeholder="留空使用默认路径 (如: ~/.codex/bin/codex)"
                @update:value="(v: string) => updateField('codexPath', v)"
              />
            </NFormItem>
            <NFormItem label="配置文件路径">
              <NInput
                :value="settingsStore.settings.codexConfigPath"
                placeholder="留空使用默认配置 (如: ~/.codex/config.toml)"
                @update:value="(v: string) => updateField('codexConfigPath', v)"
              />
            </NFormItem>
          </NForm>
          <div class="settings-hint">
            <NText depth="3" style="font-size: 12px">
              CLI 路径：指定 codex 可执行文件的完整路径。留空则自动检测。<br/>
              配置文件：指定 Codex 的配置文件路径，可用于自定义模型、API Key 等设置。
            </NText>
          </div>
        </NCard>
      </NTabPane>

      <NTabPane name="about" tab="关于">
        <NCard :bordered="false" style="background: var(--bg-secondary)">
          <NSpace vertical>
            <NText style="color: var(--text-primary)">CodeAtlas Studio v0.1.0</NText>
            <NText depth="3">统一 AI Coding Agent 工作台</NText>
          </NSpace>
        </NCard>
      </NTabPane>
    </NTabs>
  </div>
</template>

<style scoped lang="scss">
.settings-view {
  padding: 32px;
  max-width: 720px;
  margin: 0 auto;
  overflow-y: auto;
  height: 100%;
}

.settings-title {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 24px 0;
  color: var(--text-primary);
  font-family: var(--font-sans);
}

.settings-hint {
  margin-top: 8px;
  padding: 10px 12px;
  background: var(--bg-hover);
  border-radius: 8px;
  border: 0.5px solid var(--border-default);
}
</style>
