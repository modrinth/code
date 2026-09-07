import { isStaff } from '@modrinth/utils'

export default defineNuxtRouteMiddleware(async (to) => {
	const auth = await useAuth(null, to)

	if (!auth.value.user || !isStaff(auth.value.user)) {
		throw createError({
			fatal: true,
			statusCode: 401,
			statusMessage: 'Unauthorized',
		})
	}
})
