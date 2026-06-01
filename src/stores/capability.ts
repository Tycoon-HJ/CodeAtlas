import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Capability, CapabilityType } from '@/types'

export const useCapabilityStore = defineStore('capability', () => {
  const capabilities = ref<Capability[]>([])
  const loading = ref(false)

  const mcpCapabilities = computed(() =>
    capabilities.value.filter((c) => c.type === 'mcp')
  )

  const skillCapabilities = computed(() =>
    capabilities.value.filter((c) => c.type === 'skill')
  )

  async function fetchCapabilities() {
    loading.value = true
    try {
      capabilities.value = []
    } finally {
      loading.value = false
    }
  }

  async function addCapability(cap: Omit<Capability, 'id' | 'createdAt' | 'updatedAt'>) {
    const capability: Capability = {
      ...cap,
      id: crypto.randomUUID(),
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }
    capabilities.value.push(capability)
    return capability
  }

  async function toggleCapability(id: string) {
    const cap = capabilities.value.find((c) => c.id === id)
    if (cap) {
      cap.enabled = !cap.enabled
      cap.updatedAt = new Date().toISOString()
    }
  }

  async function deleteCapability(id: string) {
    capabilities.value = capabilities.value.filter((c) => c.id !== id)
  }

  return {
    capabilities,
    loading,
    mcpCapabilities,
    skillCapabilities,
    fetchCapabilities,
    addCapability,
    toggleCapability,
    deleteCapability,
  }
})
