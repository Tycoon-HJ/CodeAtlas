import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface OpenFile {
  path: string
  name: string
  content: string
  language: string
  modified: boolean
}

const LANG_MAP: Record<string, string> = {
  js: 'javascript', ts: 'typescript', tsx: 'typescript', jsx: 'javascript',
  py: 'python', rb: 'ruby', rs: 'rust', go: 'go', java: 'java',
  c: 'c', cpp: 'cpp', h: 'c', hpp: 'cpp', cs: 'csharp',
  html: 'html', htm: 'html', css: 'css', scss: 'scss', less: 'less',
  json: 'json', xml: 'xml', yaml: 'yaml', yml: 'yaml', toml: 'toml',
  md: 'markdown', sql: 'sql', sh: 'shell', bash: 'shell', zsh: 'shell',
  vue: 'html', svelte: 'html', dockerfile: 'dockerfile',
  txt: 'plaintext', log: 'plaintext', env: 'plaintext',
}

function detectLanguage(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase() ?? ''
  return LANG_MAP[ext] ?? 'plaintext'
}

export const useEditorStore = defineStore('editor', () => {
  const openFiles = ref<OpenFile[]>([])
  const activeFilePath = ref<string | null>(null)

  const activeFile = computed(() =>
    openFiles.value.find((f) => f.path === activeFilePath.value) ?? null
  )

  async function openFile(path: string) {
    const existing = openFiles.value.find((f) => f.path === path)
    if (existing) {
      activeFilePath.value = path
      return
    }

    try {
      const content = await readTextFile(path)
      const name = path.split('/').pop() ?? path
      openFiles.value.push({
        path,
        name,
        content,
        language: detectLanguage(path),
        modified: false,
      })
      activeFilePath.value = path
    } catch (e) {
      console.error('Failed to open file:', e)
    }
  }

  function closeFile(path: string) {
    const idx = openFiles.value.findIndex((f) => f.path === path)
    if (idx === -1) return
    openFiles.value.splice(idx, 1)
    if (activeFilePath.value === path) {
      activeFilePath.value = openFiles.value.length > 0
        ? openFiles.value[Math.min(idx, openFiles.value.length - 1)].path
        : null
    }
  }

  function setActive(path: string) {
    activeFilePath.value = path
  }

  function updateContent(path: string, content: string) {
    const file = openFiles.value.find((f) => f.path === path)
    if (file) {
      file.content = content
      file.modified = true
    }
  }

  function markSaved(path: string) {
    const file = openFiles.value.find((f) => f.path === path)
    if (file) file.modified = false
  }

  function closeAll() {
    openFiles.value = []
    activeFilePath.value = null
  }

  let unlistenFileChange: UnlistenFn | null = null

  async function initFileChangeListener() {
    if (unlistenFileChange) return
    unlistenFileChange = await listen<{ kind: string; paths: string[] }>('file-change', async (event) => {
      const { kind, paths } = event.payload
      if (kind === 'modify') {
        for (const changedPath of paths) {
          const file = openFiles.value.find((f) => f.path === changedPath)
          if (file && !file.modified) {
            try {
              file.content = await readTextFile(changedPath)
            } catch {
              // file may have been deleted
            }
          }
        }
      }
    })
  }

  return {
    openFiles,
    activeFilePath,
    activeFile,
    openFile,
    closeFile,
    setActive,
    updateContent,
    markSaved,
    closeAll,
    initFileChangeListener,
  }
})
