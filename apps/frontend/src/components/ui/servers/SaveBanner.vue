<script setup lang="ts">
import { SaveBanner as SharedSaveBanner } from '@modrinth/ui'

const props = defineProps<{
	isUpdating: boolean
	restart?: boolean
	save: () => void
	reset: () => void
	isVisible: boolean
	serverId: string
}>()
</script>

<template>
	<!-- TODO, delete this wrapper and update imports in /frontend accordingly to use from /ui-->
	<SharedSaveBanner v-bind="props" />
</template>
