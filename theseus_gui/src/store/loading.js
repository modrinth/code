import { defineStore } from 'pinia'

export const useLoading = defineStore('loadingStore', {
  state: () => ({ loading: false }),
  actions: {
    startLoading() {
      this.loading = true
    },
    stopLoading() {
      this.loading = false
    },
  },
})
