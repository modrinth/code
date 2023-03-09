export default defineNuxtPlugin((nuxtApp) => {
  const themeStore = useTheme()

  if (process.client && themeStore.value.preference === 'system') {
    setTimeout(() => {
      const colorSchemeQueryList = window.matchMedia('(prefers-color-scheme: light)')

      const setColorScheme = (e) => {
        if (themeStore.value.preference === 'system') {
          if (e.matches) {
            updateTheme('light')
          } else {
            updateTheme('dark')
          }
        }
      }

      setColorScheme(colorSchemeQueryList)
      colorSchemeQueryList.addEventListener('change', setColorScheme)
    }, 100)
  }

  nuxtApp.provide('colorMode', themeStore.value)
})
