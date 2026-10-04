<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageFilesPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { server, serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()
const flags = useFeatureFlags()

try {
	await queryClient.ensureQueryData({
		queryKey: ['files', serverId, '/'],
		queryFn: () => client.kyros.files_v0.listDirectory('/', 1, 2000),
		staleTime: 30_000,
	})
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}

useHead({
	title: computed(() => `Files - ${server.value?.name ?? 'Server'} - Modrinth`),
})

const collapsibleFooter = useCollapsibleFooter()
onMounted(() => {
	collapsibleFooter.value = true
})
onBeforeUnmount(() => {
	collapsibleFooter.value = false
})

defineProps<{
	constrainWidth?: boolean
}>()
</script>

<template>
	<ServersManageFilesPage
		:show-debug-info="flags.advancedDebugInfo"
		:show-refresh-button="flags.FilesRefreshButton"
		:constrain-width="constrainWidth"
	/>
</template>
