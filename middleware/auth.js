export default defineNuxtRouteMiddleware(async (_to, from) => {
  const auth = await useAuth()

  if (!auth.value.user) {
    return navigateTo(`/auth/sign-in?redirect=${encodeURIComponent(from.fullPath)}`)
  }
})
