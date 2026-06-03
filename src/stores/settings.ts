import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/api'

export type ThemeMode = 'light' | 'dark' | 'system'

export interface AppSettings {
  theme: ThemeMode
  locale: string
  gitUserName: string
  gitUserEmail: string
  proxyUrl: string
  logLevel: string
  claudePath: string
  claudeConfigPath: string
  codexPath: string
  codexConfigPath: string
  fontSize: number
  autoScroll: boolean
  autoRefresh: boolean
  terminalShell: string
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    theme: 'dark',
    locale: 'zh-CN',
    gitUserName: '',
    gitUserEmail: '',
    proxyUrl: '',
    logLevel: 'info',
    claudePath: '',
    claudeConfigPath: '',
    codexPath: '',
    codexConfigPath: '',
    fontSize: 14,
    autoScroll: true,
    autoRefresh: true,
    terminalShell: '',
  })

  async function loadSettings() {
    try {
      const saved = await api.loadSettings()
      if (saved) {
        settings.value = { ...settings.value, ...saved }
      }
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }

  async function saveSettings() {
    try {
      await api.saveSettings(settings.value)
    } catch (e) {
      console.error('Failed to save settings:', e)
    }
  }

  function updateSettings(partial: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...partial }
    saveSettings()
  }

  function setTheme(theme: ThemeMode) {
    settings.value.theme = theme
    saveSettings()
  }

  return {
    settings,
    loadSettings,
    saveSettings,
    updateSettings,
    setTheme,
  }
})
