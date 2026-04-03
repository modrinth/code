const COLLECTION_PREFIX = '/collection/bLYCa4PJ'

export default defineNuxtRouteMiddleware((to) => {
	if (to.path === COLLECTION_PREFIX || to.path.startsWith(`${COLLECTION_PREFIX}/`)) {
		return navigateTo(`https://april-fools-2026.modrinth.com${to.fullPath}`, {
			external: true,
			redirectCode: 302,
		})
	}
})
