import type { Archon } from '@modrinth/api-client'
import { useMutation, useQueryClient } from '@tanstack/vue-query'

import { injectModrinthClient } from '../providers'

export function useDismissServerIntro() {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()

	return useMutation({
		mutationFn: (serverId: string) => client.archon.servers_v1.endIntro(serverId),
		onSuccess: async (_result, serverId) => {
			const detailKey = ['servers', 'detail', serverId] as const
			await Promise.all([
				queryClient.cancelQueries({ queryKey: detailKey }),
				queryClient.cancelQueries({ queryKey: ['servers'], exact: true }),
			])
			queryClient.setQueryData<Archon.Servers.v0.Server>(detailKey, (server) =>
				server ? { ...server, flows: { ...server.flows, intro: false } } : server,
			)
			queryClient.setQueryData<Archon.Servers.v0.ServerGetResponse>(['servers'], (data) =>
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
			void queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] })
			void queryClient.invalidateQueries({ queryKey: ['servers'], exact: true })
		},
	})
}
