import { ref } from 'vue'

export type LibraryTab = 'installed' | 'available'

export const librarySearch = ref('')
export const libraryTab = ref<LibraryTab>('installed')
export const libraryScrollTop = ref(0)
