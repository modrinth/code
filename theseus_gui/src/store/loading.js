import { defineStore } from 'pinia'

export const useLoading = defineStore('loadingStore', {
  state: () => ({ loading: false }),
  actions: {
    startLoading() {
      console.log('I was told to start loading')
      this.loading = true
    },
    stopLoading() {
      console.log('I was told to stop loading')
      this.loading = false
    },
  },
})
