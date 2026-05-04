import { isStaff } from '@modrinth/utils'

export default defineNuxtRouteMiddleware(async () => {
	const auth = await useAuth()

	if (!auth.value.user || !isStaff(auth.value.user)) {
		throw createError({
			fatal: true,
			statusCode: 401,
			statusMessage: 'Unauthorized',
		})
	}
})
