import { useAppQueryClient } from '~/composables/query-client'
import { useServerModrinthClient } from '~/server/utils/api-client'

export default defineNuxtRouteMiddleware(async (to) => {
	if (!to.path.startsWith('/user/') || !to.params.id) {
		return
	}

	const queryClient = useAppQueryClient()
	const authToken = useCookie('auth-token')
	const client = useServerModrinthClient({ authToken: authToken.value || undefined })
	const userId = to.params.id as string

	try {
		const user = await queryClient.fetchQuery({
			queryKey: ['user', userId],
			queryFn: () => client.labrinth.users_v2.get(userId),
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
