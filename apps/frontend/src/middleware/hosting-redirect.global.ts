export default defineNuxtRouteMiddleware((to) => {
	if (to.path.startsWith('/servers')) {
		const target = to.path.replace('/servers', '/hosting')
		return navigateTo(target, { redirectCode: 301 })
	}
})
