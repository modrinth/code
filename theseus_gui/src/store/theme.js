import { defineStore } from 'pinia'
import { setDarkTheme, setLightTheme } from '@/helpers/theme'

export const useTheming = defineStore('themeStore', {
  state: () => ({ themeOptions: ['light', 'dark'], selectedTheme: 'dark', darkTheme: true }),
  actions: {
    setThemeState(newTheme) {
      if (this.themeOptions.includes(newTheme)) this.selectedTheme = newTheme
      else console.warn('Selected theme is not present. Check themeOptions.')

      this.setThemeClass()
    },
    setThemeClass() {
      if (this.selectedTheme === 'dark') setDarkTheme()
      else if (this.selectedTheme === 'light') setLightTheme()
    },
  },
})
