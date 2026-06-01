import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Provider, ProviderType, ProviderCapabilities } from '@/types'
import * as api from '@/api'

const defaultCaps: ProviderCapabilities = {
  supportsCli: true,
  supportsMcp: true,
  supportsResume: true,
  supportsSlashCommand: true,
  supportsVision: false,
  supportsAgent: false,
}

const providerDefaults: Record<string, { name: string; type: ProviderType; caps: Partial<ProviderCapabilities> }> = {
  claude: { name: 'Claude Code', type: 'claude', caps: { supportsVision: true, supportsAgent: true } },
  codex: { name: 'Codex CLI', type: 'codex', caps: { supportsResume: false } },
  gemini: { name: 'Gemini CLI', type: 'gemini', caps: { supportsResume: false, supportsVision: true } },
  qwen: { name: 'Qwen-Code', type: 'qwen', caps: { supportsResume: false } },
  opencode: { name: 'OpenCode', type: 'opencode', caps: { supportsResume: false } },
}

export const useAgentStore = defineStore('agent', () => {
  const providers = ref<Provider[]>(
    Object.entries(providerDefaults).map(([id, def]) => ({
      id,
      type: def.type,
      name: def.name,
      enabled: id === 'claude',
      config: {},
      capabilities: { ...defaultCaps, ...def.caps },
    }))
  )
  const currentProvider = ref<Provider | null>(providers.value[0])
  const loading = ref(false)

  async function syncProviderAvailability() {
    try {
      const backendProviders = await api.listProviders()
      for (const bp of backendProviders) {
        const p = providers.value.find((pr) => pr.id === bp.id)
        if (p) {
          p.name = bp.name
          // If backend says unavailable, disable it
          if (!bp.available && p.enabled && p.id !== 'claude') {
            p.enabled = false
          }
        }
      }
    } catch {
      // Backend doesn't support list_providers yet, ignore
    }
  }

  function setCurrentProvider(provider: Provider | null) {
    currentProvider.value = provider
  }

  function toggleProvider(id: string) {
    const provider = providers.value.find((p) => p.id === id)
    if (provider) {
      provider.enabled = !provider.enabled
    }
  }

  function updateProviderConfig(id: string, config: Record<string, unknown>) {
    const provider = providers.value.find((p) => p.id === id)
    if (provider) {
      provider.config = { ...provider.config, ...config }
    }
  }

  function getEnabledProviders(): Provider[] {
    return providers.value.filter((p) => p.enabled)
  }

  return {
    providers,
    currentProvider,
    loading,
    syncProviderAvailability,
    setCurrentProvider,
    toggleProvider,
    updateProviderConfig,
    getEnabledProviders,
  }
})
