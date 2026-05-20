export default defineNuxtRouteMiddleware((to) => {
	if (to.path.startsWith('/flags')) {
		const target = to.fullPath.replace('/flags', '/settings/flags')
		return navigateTo(target, { redirectCode: 301 })
	}
})
