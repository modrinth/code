<template>
	<div
		v-tooltip="showCustomModpackTooltip ? 'This project uses a custom modpack' : name"
		class="flex gap-1.5 items-center flex-shrink overflow-hidden smart-clickable:allow-pointer-events"
		:class="[onclick ? 'hover:underline cursor-pointer' : '']"
		@click="onclick"
	>
		<Avatar :src="icon" size="24px" />
		<span class="truncate font-medium">
			{{ name }}
		</span>
	</div>
</template>

<script setup lang="ts">
import Avatar from '../../base/Avatar.vue'

defineProps<{
	name: string
	icon?: string
	onclick?: () => void
	showCustomModpackTooltip?: boolean
}>()
</script>
