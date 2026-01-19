<script setup lang="ts">
import { injectModrinthServerContext, ServersManageContentOldPage } from '@modrinth/ui'

import { useGeneratedState } from '~/composables/generated'

const { server } = injectModrinthServerContext()
const generatedState = useGeneratedState()

useHead({
	title: `Content - ${server.value?.name ?? 'Server'} - Modrinth`,
})

const tags = computed(() => ({
	gameVersions: generatedState.value.gameVersions,
	loaders: generatedState.value.loaders,
}))
</script>

<template>
	<ServersManageContentOldPage :tags="tags" />
</template>
