<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageFilesPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { worldId } = injectModrinthServerContext()
const queryClient = useQueryClient()

try {
	if (worldId.value) {
		await queryClient.ensureQueryData({
			queryKey: ['files', 'v1', worldId.value, '/'],
			queryFn: () => client.kyros.files_v1.listDescendants(worldId.value!, '/', 1, 200),
			staleTime: 30_000,
		})
	}
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}
</script>

<template>
	<ServersManageFilesPage />
</template>
