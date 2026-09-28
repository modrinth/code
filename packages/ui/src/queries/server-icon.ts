import { type AbstractModrinthClient, ModrinthApiError } from '@modrinth/api-client'
import { queryOptions } from '@tanstack/vue-query'

export function serverIconQueryOptions(serverId: string, client: AbstractModrinthClient) {
	return queryOptions({
		queryKey: ['server-icon', serverId] as const,
		queryFn: async (): Promise<string | null> => {
			try {
				const blob = await client.archon.icons_v1.get(serverId)
				const bytes = new Uint8Array(await blob.arrayBuffer())
				const binary = bytes.reduce((value, byte) => value + String.fromCharCode(byte), '')
				return `data:image/png;base64,${btoa(binary)}`
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 404) return null
				throw error
			}
		},
	})
}
