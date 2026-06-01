<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, shallowRef } from 'vue'
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker'
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker'
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker'
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker'
import * as monaco from 'monaco-editor'

// Configure Monaco workers for Vite
self.MonacoEnvironment = {
  getWorker(_, label) {
    if (label === 'json') return new jsonWorker()
    if (label === 'css' || label === 'scss' || label === 'less') return new cssWorker()
    if (label === 'html' || label === 'handlebars' || label === 'razor') return new htmlWorker()
    if (label === 'typescript' || label === 'javascript') return new tsWorker()
    return new editorWorker()
  },
}

const props = defineProps<{
  content: string
  language: string
  readOnly?: boolean
}>()

const emit = defineEmits<{
  'update:content': [value: string]
}>()

const containerRef = ref<HTMLDivElement | null>(null)
const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)

onMounted(() => {
  if (!containerRef.value) return

  editor.value = monaco.editor.create(containerRef.value, {
    value: props.content,
    language: props.language,
    readOnly: props.readOnly ?? false,
    theme: 'vs-dark',
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 13,
    lineHeight: 20,
    padding: { top: 12, bottom: 12 },
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    tabSize: 2,
    renderLineHighlight: 'line',
    bracketPairColorization: { enabled: true },
    smoothScrolling: true,
    cursorBlinking: 'smooth',
    cursorSmoothCaretAnimation: 'on',
  })

  editor.value.onDidChangeModelContent(() => {
    emit('update:content', editor.value?.getValue() ?? '')
  })
})

watch(() => props.content, (newVal) => {
  if (editor.value && editor.value.getValue() !== newVal) {
    editor.value.setValue(newVal)
  }
})

watch(() => props.language, (newVal) => {
  if (editor.value) {
    const model = editor.value.getModel()
    if (model) monaco.editor.setModelLanguage(model, newVal)
  }
})

watch(() => props.readOnly, (newVal) => {
  editor.value?.updateOptions({ readOnly: newVal ?? false })
})

onBeforeUnmount(() => {
  editor.value?.dispose()
})
</script>

<template>
  <div ref="containerRef" class="code-editor" />
</template>

<style scoped lang="scss">
.code-editor {
  width: 100%;
  height: 100%;
  min-height: 200px;
}
</style>
