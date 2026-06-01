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

const themeOverrides = {
  common: {
    primaryColor: '#0A84FF',
    primaryColorHover: '#409CFF',
    primaryColorPressed: '#0070E0',
    borderRadius: '8px',
    borderRadiusSmall: '6px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Arial, sans-serif',
    fontFamilyMono: '"SF Mono", "Menlo", "Monaco", "Courier New", monospace',
    bodyColor: '#1C1C1E',
    cardColor: '#2C2C2E',
    modalColor: '#2C2C2E',
    popoverColor: '#3A3A3C',
    tableColor: '#1C1C1E',
    inputColor: '#1C1C1E',
    actionColor: '#2C2C2E',
    tagColor: '#3A3A3C',
    hoverColor: 'rgba(255, 255, 255, 0.05)',
    borderColor: 'rgba(255, 255, 255, 0.1)',
    dividerColor: 'rgba(255, 255, 255, 0.06)',
    textColorBase: '#FFFFFF',
    textColor1: '#FFFFFF',
    textColor2: '#98989D',
    textColor3: '#636366',
    placeholderColor: '#636366',
    iconColor: '#98989D',
    iconColorHover: '#FFFFFF',
    closeIconColor: '#98989D',
    closeIconColorHover: '#FFFFFF',
  },
  Card: {
    borderRadius: '10px',
    color: '#2C2C2E',
    borderColor: 'rgba(255, 255, 255, 0.06)',
  },
  Input: {
    color: 'rgba(255, 255, 255, 0.06)',
    border: '1px solid rgba(255, 255, 255, 0.1)',
    borderHover: '1px solid #0A84FF',
    borderFocus: '1px solid #0A84FF',
    borderRadius: '8px',
  },
  Button: {
    borderRadiusMedium: '8px',
    borderRadiusSmall: '6px',
  },
  Tag: {
    borderRadius: '6px',
  },
  Menu: {
    borderRadius: '8px',
    color: 'transparent',
    itemColorActive: 'rgba(10, 132, 255, 0.15)',
    itemColorActiveHover: 'rgba(10, 132, 255, 0.2)',
    itemTextColorActive: '#FFFFFF',
    itemIconColorActive: '#0A84FF',
  },
  Breadcrumb: {
    separatorColor: '#636366',
    itemTextColor: '#98989D',
    itemTextColorHover: '#FFFFFF',
    itemTextColorActive: '#FFFFFF',
  },
  Scrollbar: {
    color: 'rgba(255, 255, 255, 0.15)',
    colorHover: 'rgba(255, 255, 255, 0.25)',
  },
  Tabs: {
    tabTextColorLine: '#98989D',
    tabTextColorActiveLine: '#FFFFFF',
    tabTextColorHoverLine: '#FFFFFF',
    barColorLine: '#0A84FF',
  },
  Select: {
    peers: {
      InternalSelection: {
        borderRadius: '8px',
      },
    },
  },
}
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
