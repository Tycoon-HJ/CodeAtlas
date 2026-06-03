<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, zhCN, dateZhCN, darkTheme } from 'naive-ui'
import { computed, watch, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()

function applyTheme(themeMode: string) {
  const isDark =
    themeMode === 'dark' ||
    (themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light')
}

onMounted(() => {
  applyTheme(settingsStore.settings.theme)
})

watch(() => settingsStore.settings.theme, (v) => applyTheme(v))

const theme = computed(() => {
  const t = settingsStore.settings.theme
  const isDark =
    t === 'dark' || (t === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  return isDark ? darkTheme : null
})

const themeOverrides = computed(() => {
  const t = settingsStore.settings.theme
  const isDark =
    t === 'dark' || (t === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)

  return {
    common: {
      primaryColor: isDark ? '#3B82F6' : '#2563EB',
      primaryColorHover: isDark ? '#2563EB' : '#1D4ED8',
      primaryColorPressed: isDark ? '#1D4ED8' : '#1E40AF',
      borderRadius: '6px',
      borderRadiusSmall: '4px',
      fontFamily: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
      fontFamilyMono: '"JetBrains Mono", "Fira Code", "Consolas", monospace',
      bodyColor: isDark ? '#111827' : '#F7F8FA',
      cardColor: isDark ? '#1E293B' : '#FFFFFF',
      modalColor: isDark ? '#1E293B' : '#FFFFFF',
      popoverColor: isDark ? '#1E293B' : '#FFFFFF',
      tableColor: isDark ? '#1E293B' : '#FFFFFF',
      inputColor: isDark ? '#1E293B' : '#FFFFFF',
      actionColor: isDark ? '#1E293B' : '#F7F8FA',
      tagColor: isDark ? '#1E293B' : '#F1F3F5',
      hoverColor: isDark ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.04)',
      borderColor: isDark ? '#2D3748' : '#E1E4E8',
      dividerColor: isDark ? '#2D3748' : '#E1E4E8',
      textColorBase: isDark ? '#F8FAFC' : '#1F2328',
      textColor1: isDark ? '#F8FAFC' : '#1F2328',
      textColor2: isDark ? '#94A3B8' : '#59636E',
      textColor3: isDark ? '#64748B' : '#8B949E',
      placeholderColor: isDark ? '#475569' : '#AFB8C1',
      iconColor: isDark ? '#94A3B8' : '#59636E',
      iconColorHover: isDark ? '#F8FAFC' : '#1F2328',
      closeIconColor: isDark ? '#94A3B8' : '#59636E',
      closeIconColorHover: isDark ? '#F8FAFC' : '#1F2328',
    },
    Card: {
      borderRadius: '8px',
      color: isDark ? '#1E293B' : '#FFFFFF',
      borderColor: isDark ? '#2D3748' : '#E1E4E8',
    },
    Input: {
      color: isDark ? '#1E293B' : '#FFFFFF',
      border: `1px solid ${isDark ? '#2D3748' : '#E1E4E8'}`,
      borderHover: `1px solid ${isDark ? '#3B82F6' : '#2563EB'}`,
      borderFocus: `1px solid ${isDark ? '#3B82F6' : '#2563EB'}`,
      borderRadius: '6px',
    },
    Button: {
      borderRadiusMedium: '6px',
      borderRadiusSmall: '4px',
    },
    Tag: {
      borderRadius: '4px',
    },
    Menu: {
      borderRadius: '6px',
      color: 'transparent',
      itemColorActive: isDark ? 'rgba(59, 130, 246, 0.15)' : 'rgba(37, 99, 235, 0.1)',
      itemColorActiveHover: isDark ? 'rgba(59, 130, 246, 0.2)' : 'rgba(37, 99, 235, 0.15)',
      itemTextColorActive: isDark ? '#3B82F6' : '#2563EB',
      itemIconColorActive: isDark ? '#3B82F6' : '#2563EB',
    },
    Breadcrumb: {
      separatorColor: isDark ? '#64748B' : '#8B949E',
      itemTextColor: isDark ? '#94A3B8' : '#59636E',
      itemTextColorHover: isDark ? '#F8FAFC' : '#1F2328',
      itemTextColorActive: isDark ? '#F8FAFC' : '#1F2328',
    },
    Scrollbar: {
      color: isDark ? 'rgba(255, 255, 255, 0.15)' : 'rgba(0, 0, 0, 0.15)',
      colorHover: isDark ? 'rgba(255, 255, 255, 0.25)' : 'rgba(0, 0, 0, 0.25)',
    },
    Tabs: {
      tabTextColorLine: isDark ? '#94A3B8' : '#59636E',
      tabTextColorActiveLine: isDark ? '#F8FAFC' : '#1F2328',
      tabTextColorHoverLine: isDark ? '#F8FAFC' : '#1F2328',
      barColorLine: isDark ? '#3B82F6' : '#2563EB',
    },
    Select: {
      peers: {
        InternalSelection: {
          borderRadius: '6px',
        },
      },
    },
  }
})
</script>

<template>
  <NConfigProvider :theme="theme" :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <NDialogProvider>
        <RouterView />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
