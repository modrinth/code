export default defineNuxtPlugin(async (nuxtApp) => {
  const authStore = await useAuth()
  await useUser()
  const cosmeticsStore = useCosmetics()
  const tagsStore = useTags()

  nuxtApp.provide('auth', authStore.value)
  nuxtApp.provide('cosmetics', cosmeticsStore.value)
  nuxtApp.provide('tag', tagsStore.value)
  nuxtApp.provide('notify', (notif) => addNotification(notif))
})
