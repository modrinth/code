import { invoke } from '@tauri-apps/api/tauri'

export default {
  async mounted() {
    await this.loadCssFromRustBackend()
  },
  methods: {
    async loadCssFromRustBackend() {
      try {
        // Fetch the CSS content from the Rust backend
        const cssContent = await invoke('get_css')

        // Create a style element and set its content
        const styleElement = document.createElement('style')
        styleElement.innerHTML = cssContent

        // Append the style element to the document's head
        document.head.appendChild(styleElement)
      } catch (error) {
        console.error('Error loading CSS from Rust backend:', error)
      }
    },
  },
}
