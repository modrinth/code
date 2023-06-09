import { defineStore } from 'pinia'

export const useTheming = defineStore('themeStore', {
  state: () => ({
    themeOptions: ['dark'],
    collapsedNavigation: false,
    selectedTheme: 'dark',
    darkTheme: true,
  }),
  actions: {
    setThemeState(newTheme) {
      if (this.themeOptions.includes(newTheme)) this.selectedTheme = newTheme
      else console.warn('Selected theme is not present. Check themeOptions.')

      this.setThemeClass()
    },
    setThemeClass() {
      document.getElementsByTagName('html')[0].classList.remove('dark-mode')
      document.getElementsByTagName('html')[0].classList.remove('light-mode')
      document.getElementsByTagName('html')[0].classList.add(`${this.selectedTheme}-mode`)
    },
  },
})
