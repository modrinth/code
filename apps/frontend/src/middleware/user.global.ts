import { useAppQueryClient } from '~/composables/query-client'
import { useServerModrinthClient } from '~/server/utils/api-client'

export default defineNuxtRouteMiddleware(async (to) => {
	const userParam = to.params.user ?? to.params.id
	const userId = Array.isArray(userParam) ? userParam[0] : userParam

	if (!to.path.startsWith('/user/') || !userId) {
		return
	}

	const queryClient = useAppQueryClient()
	const authToken = useCookie('auth-token')
	const client = useServerModrinthClient({ authToken: authToken.value || undefined })

	try {
		const user = await queryClient.fetchQuery({
			queryKey: ['user', userId],
			queryFn: () => client.labrinth.users_v3.get(userId),
		})

		if (!user) return

		if (user.username !== userId) {
			return navigateTo(`/user/${user.username}`, {
				redirectCode: 301,
				replace: true,
			})
		}
	} catch {
		// Let the page handle 404s and other errors
	}
})
