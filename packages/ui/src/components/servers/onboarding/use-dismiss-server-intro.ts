import type { Archon } from '@modrinth/api-client'
import { useMutation, useQueryClient } from '@tanstack/vue-query'

import { serverKeys } from '#ui/composables/server-queries'
import { injectModrinthClient } from '#ui/providers'

export function useDismissServerIntro() {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()

	return useMutation({
		mutationFn: (serverId: string) => client.archon.servers_v1.endIntro(serverId),
		onSuccess: async (_result, serverId) => {
			const detailKey = serverKeys.detail(serverId)
			await Promise.all([
				queryClient.cancelQueries({ queryKey: detailKey }),
				queryClient.cancelQueries({ queryKey: serverKeys.list(), exact: true }),
			])
			queryClient.setQueryData<Archon.Servers.v0.Server>(detailKey, (server) =>
				server ? { ...server, flows: { ...server.flows, intro: false } } : server,
			)
			queryClient.setQueryData<Archon.Servers.v0.ServerGetResponse>(serverKeys.list(), (data) =>
				data
					? {
							...data,
							servers: data.servers.map((server) =>
								server.server_id === serverId
									? { ...server, flows: { ...server.flows, intro: false } }
									: server,
							),
						}
					: data,
			)
			void queryClient.invalidateQueries({ queryKey: detailKey })
			void queryClient.invalidateQueries({ queryKey: serverKeys.detailV1(serverId) })
			void queryClient.invalidateQueries({ queryKey: serverKeys.list(), exact: true })
		},
	})
}
