import { defineStore } from 'pinia'

export const useLoading = defineStore('loadingStore', {
  state: () => ({
    loading: false,
    barEnabled: false,
  }),
  actions: {
    setEnabled(enabled) {
      this.barEnabled = enabled
    },
    startLoading() {
      this.loading = true
    },
    stopLoading() {
      this.loading = false
    },
  },
})
