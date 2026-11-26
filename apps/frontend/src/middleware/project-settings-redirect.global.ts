export default defineNuxtRouteMiddleware((to) => {
	// redirect /:user/:project/settings/license to /:user/:project/settings#license
	const settingsPath = to.path.match(/^(\/[^/]+\/[^/]+\/settings)\/license\/?$/)?.[1]
	if (!settingsPath) return

	return navigateTo(
		{
			path: settingsPath,
			hash: to.hash || '#license',
			query: to.query,
		},
		{ replace: true, redirectCode: 301 },
	)
})
