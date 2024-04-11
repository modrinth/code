export default defineNuxtPlugin(async (nuxtApp) => {
  await useAuth()
  await useUser()
  const themeStore = useTheme()
  const cosmetics = useCosmetics()

  nuxtApp.hook('app:mounted', () => {
    if (process.client && themeStore.value.preference === 'system') {
      const colorSchemeQueryList = window.matchMedia('(prefers-color-scheme: light)')

      const setColorScheme = (e) => {
        if (themeStore.value.preference === 'system') {
          if (e.matches) {
            updateTheme('light')
          } else {
            updateTheme(cosmetics.value.preferredDarkTheme ?? 'dark')
          }
        }
      }

      setColorScheme(colorSchemeQueryList)
      colorSchemeQueryList.addEventListener('change', setColorScheme)
    }
  })

  nuxtApp.provide('colorMode', themeStore.value)
})
