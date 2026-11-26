import { defineAsyncComponent } from 'vue'

export const MarkdownEditor = defineAsyncComponent(() => import('./index.vue'))
export { default as MarkdownEditorToolbar } from './toolbar.vue'
export type { MarkdownEditorMode } from './types'
