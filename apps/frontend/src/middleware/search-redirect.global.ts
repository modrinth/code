export default defineNuxtRouteMiddleware((to) => {
	if (
		to.path.startsWith('/mods') ||
		to.path.startsWith('/modpacks') ||
		to.path.startsWith('/plugins') ||
		to.path.startsWith('/datapacks') ||
		to.path.startsWith('/resourcepacks') ||
		to.path.startsWith('/shaders')
	) {
		const target = '/discover' + to.fullPath
		return navigateTo(target, { redirectCode: 301 })
	}
})
