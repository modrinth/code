import { defineStore } from 'pinia'

export const useLogs = defineStore('logStore', {
  state: () => ({
    logsColored: null,
  }),
})
