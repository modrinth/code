export default defineNuxtPlugin(async (nuxtApp) => {
  const authStore = await useAuth()
  const userStore = await useUser(true)
  const cosmeticsStore = useCosmetics()
  const tagsStore = useTags()

  nuxtApp.provide('auth', authStore.value)
  nuxtApp.provide('user', userStore)
  nuxtApp.provide('cosmetics', cosmeticsStore.value)
  nuxtApp.provide('tag', tagsStore.value)
  nuxtApp.provide('notify', (notif) => addNotification(notif))
})
