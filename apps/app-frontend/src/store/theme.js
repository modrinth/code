import { defineStore } from 'pinia'

export const useTheming = defineStore('themeStore', {
  state: () => ({
    themeOptions: ['dark', 'light', 'oled', 'system'],
    advancedRendering: true,
    selectedTheme: 'dark',
    toggleSidebar: false,

    devMode: false,
    featureFlags: {},
  }),
  actions: {
    setThemeState(newTheme) {
      if (this.themeOptions.includes(newTheme)) this.selectedTheme = newTheme
      else console.warn('Selected theme is not present. Check themeOptions.')

      this.setThemeClass()
    },
    setThemeClass() {
      for (const theme of this.themeOptions) {
        document.getElementsByTagName('html')[0].classList.remove(`${theme}-mode`)
      }

      let theme = this.selectedTheme
      if (this.selectedTheme === 'system') {
        const darkThemeMq = window.matchMedia('(prefers-color-scheme: dark)')
        if (darkThemeMq.matches) {
          theme = 'dark'
        } else {
          theme = 'light'
        }
      }

      document.getElementsByTagName('html')[0].classList.add(`${theme}-mode`)
    },
  },
})
