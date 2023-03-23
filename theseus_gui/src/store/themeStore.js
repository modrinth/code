import { defineStore } from 'pinia'

export const useTheming = defineStore('themeStore', {
  state: () => ({ darkTheme: true }),
  actions: {
    toggleTheme() {
      this.darkTheme = !this.darkTheme
    },
  },
})
