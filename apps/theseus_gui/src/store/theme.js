import { defineStore } from 'pinia'

export const useTheming = defineStore('themeStore', {
  state: () => ({
    themeOptions: ['dark', 'light', 'oled'],
    advancedRendering: true,
    selectedTheme: 'dark',
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
      document.getElementsByTagName('html')[0].classList.add(`${this.selectedTheme}-mode`)
    },
  },
})
