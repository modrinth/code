import { defineStore } from 'pinia'

export const DEFAULT_FEATURE_FLAGS = {
  project_background: false,
  page_path: false,
  worlds_tab: false,
  worlds_in_home: true,
}

export const THEME_OPTIONS = ['dark', 'light', 'oled', 'system'] as const

export type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS
export type FeatureFlags = Record<FeatureFlag, boolean>
export type ColorTheme = (typeof THEME_OPTIONS)[number]

export type ThemeStore = {
  selectedTheme: ColorTheme
  advancedRendering: boolean
  toggleSidebar: boolean

  devMode: boolean
  featureFlags: FeatureFlags
}

export const DEFAULT_THEME_STORE: ThemeStore = {
  selectedTheme: 'dark',
  advancedRendering: true,
  toggleSidebar: false,

  devMode: false,
  featureFlags: DEFAULT_FEATURE_FLAGS,
}

export const useTheming = defineStore('themeStore', {
  state: () => DEFAULT_THEME_STORE,
  actions: {
    setThemeState(newTheme: ColorTheme) {
      if (THEME_OPTIONS.includes(newTheme)) {
        this.selectedTheme = newTheme
      } else {
        console.warn('Selected theme is not present. Check themeOptions.')
      }

      this.setThemeClass()
    },
    setThemeClass() {
      for (const theme of THEME_OPTIONS) {
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
    getFeatureFlag(key: FeatureFlag) {
      return this.featureFlags[key] ?? DEFAULT_FEATURE_FLAGS[key]
    },
    getThemeOptions() {
      return THEME_OPTIONS
    },
  },
})
