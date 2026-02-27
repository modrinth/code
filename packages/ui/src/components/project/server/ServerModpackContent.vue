<template>
	<div
		v-tooltip="showCustomModpackTooltip ? 'This project uses a custom modpack' : undefined"
		class="flex gap-1.5 items-center"
		:class="[
			onclick ? 'hover:underline cursor-pointer' : '',
			onclick || showCustomModpackTooltip ? 'smart-clickable:allow-pointer-events' : '',
		]"
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
