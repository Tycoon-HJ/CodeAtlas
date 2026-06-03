<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { NText, NSpin, NEmpty } from 'naive-ui'
import { readDir, stat } from '@tauri-apps/plugin-fs'

const props = defineProps<{
  rootPath: string
}>()

const emit = defineEmits<{
  'file-click': [path: string]
}>()

interface TreeNode {
  name: string
  path: string
  isDirectory: boolean
  children?: TreeNode[]
  expanded?: boolean
  loaded?: boolean
}

const tree = ref<TreeNode[]>([])
const loading = ref(false)
const error = ref('')

// Common directories to skip
const SKIP_DIRS = new Set([
  'node_modules', '.git', 'target', 'dist', '.next', '__pycache__', '.venv', 'venv',
  '.idea', '.vscode', '.DS_Store', 'Thumbs.db',
])

const ICONS: Record<string, string> = {
  folder: '📁',
  file: '📄',
  ts: '🟦',
  tsx: '🟦',
  js: '🟨',
  jsx: '🟨',
  rs: '🦀',
  vue: '💚',
  json: '📋',
  md: '📝',
  html: '🌐',
  css: '🎨',
  scss: '🎨',
  py: '🐍',
  go: '🔷',
  toml: '⚙️',
  yaml: '⚙️',
  yml: '⚙️',
  gitignore: '🙈',
  lock: '🔒',
}

function getIcon(node: TreeNode): string {
  if (node.isDirectory) return ICONS.folder
  const ext = node.name.split('.').pop()?.toLowerCase() ?? ''
  if (ICONS[ext]) return ICONS[ext]
  if (node.name.startsWith('.')) return ICONS.gitignore ?? ICONS.file
  return ICONS.file
}

async function loadChildren(node: TreeNode) {
  if (node.loaded) return
  try {
    const entries = await readDir(node.path)
    const children: TreeNode[] = []
    for (const entry of entries) {
      if (SKIP_DIRS.has(entry.name)) continue
      const childPath = `${node.path}/${entry.name}`
      children.push({
        name: entry.name,
        path: childPath,
        isDirectory: entry.isDirectory,
        expanded: false,
        loaded: false,
      })
    }
    // Sort: directories first, then alphabetically
    children.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    node.children = children
    node.loaded = true
  } catch (e: any) {
    console.error(`Failed to read dir ${node.path}:`, e)
  }
}

async function toggleExpand(node: TreeNode) {
  if (!node.isDirectory) {
    emit('file-click', node.path)
    return
  }
  node.expanded = !node.expanded
  if (node.expanded && !node.loaded) {
    await loadChildren(node)
  }
}

async function loadRoot() {
  if (!props.rootPath) return
  loading.value = true
  error.value = ''
  try {
    const entries = await readDir(props.rootPath)
    const nodes: TreeNode[] = []
    for (const entry of entries) {
      if (SKIP_DIRS.has(entry.name)) continue
      const childPath = `${props.rootPath}/${entry.name}`
      nodes.push({
        name: entry.name,
        path: childPath,
        isDirectory: entry.isDirectory,
        expanded: false,
        loaded: false,
      })
    }
    nodes.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    tree.value = nodes
  } catch (e: any) {
    error.value = e?.message ?? String(e)
  } finally {
    loading.value = false
  }
}

watch(() => props.rootPath, () => loadRoot(), { immediate: true })
</script>

<template>
  <div class="file-tree">
    <div v-if="loading" class="tree-loading">
      <NSpin size="small" />
    </div>
    <div v-else-if="error" class="tree-error">
      <NText depth="3" style="font-size: 12px">{{ error }}</NText>
    </div>
    <div v-else-if="tree.length === 0" class="tree-empty">
      <NText depth="3" style="font-size: 12px">空目录</NText>
    </div>
    <div v-else class="tree-nodes">
      <template v-for="node in tree" :key="node.path">
        <div
          class="tree-node"
          :class="{ directory: node.isDirectory, expanded: node.expanded }"
          @click="toggleExpand(node)"
        >
          <span class="expand-icon">{{ node.isDirectory ? (node.expanded ? '▼' : '▶') : '' }}</span>
          <span class="node-icon">{{ getIcon(node) }}</span>
          <span class="node-name">{{ node.name }}</span>
        </div>
        <div v-if="node.isDirectory && node.expanded && node.children" class="tree-children">
          <template v-for="child in node.children" :key="child.path">
            <div
              class="tree-node child"
              :class="{ directory: child.isDirectory, expanded: child.expanded }"
              @click="toggleExpand(child)"
            >
              <span class="expand-icon">{{ child.isDirectory ? (child.expanded ? '▼' : '▶') : '' }}</span>
              <span class="node-icon">{{ getIcon(child) }}</span>
              <span class="node-name">{{ child.name }}</span>
            </div>
            <div v-if="child.isDirectory && child.expanded && child.children" class="tree-children">
              <template v-for="gc in child.children" :key="gc.path">
                <div
                  class="tree-node grandchild"
                  :class="{ directory: gc.isDirectory, expanded: gc.expanded }"
                  @click="toggleExpand(gc)"
                >
                  <span class="expand-icon">{{ gc.isDirectory ? (gc.expanded ? '▼' : '▶') : '' }}</span>
                  <span class="node-icon">{{ getIcon(gc) }}</span>
                  <span class="node-name">{{ gc.name }}</span>
                </div>
                <div v-if="gc.isDirectory && gc.expanded && gc.children" class="tree-children">
                  <div
                    v-for="ggc in gc.children"
                    :key="ggc.path"
                    class="tree-node deep"
                    @click="toggleExpand(ggc)"
                  >
                    <span class="expand-icon">{{ ggc.isDirectory ? (ggc.expanded ? '▼' : '▶') : '' }}</span>
                    <span class="node-icon">{{ getIcon(ggc) }}</span>
                    <span class="node-name">{{ ggc.name }}</span>
                  </div>
                </div>
              </template>
            </div>
          </template>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped lang="scss">
.file-tree {
  font-size: 13px;
  user-select: none;
}

.tree-loading, .tree-error, .tree-empty {
  padding: 12px 16px;
  text-align: center;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px 3px 12px;
  cursor: pointer;
  border-radius: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover {
    background: var(--bg-hover);
  }

  &.child {
    padding-left: 28px;
  }

  &.grandchild {
    padding-left: 44px;
  }

  &.deep {
    padding-left: 60px;
  }
}

.expand-icon {
  width: 14px;
  font-size: 10px;
  color: var(--text-tertiary);
  flex-shrink: 0;
  text-align: center;
}

.node-icon {
  flex-shrink: 0;
  font-size: 14px;
}

.node-name {
  overflow: hidden;
  text-overflow: ellipsis;
}

.tree-children {
  // Nested children
}
</style>
