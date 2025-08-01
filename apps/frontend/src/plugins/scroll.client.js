export default defineNuxtPlugin(() => {
  const nuxtApp = useNuxtApp()

  nuxtApp.hooks.hook('page:transition:finish', () => {
    document.querySelector('[data-scroll]')?.scrollTo({ top: 0 })
  })
})
