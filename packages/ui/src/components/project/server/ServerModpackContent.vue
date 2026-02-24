<template>
	<div
		class="flex gap-1.5 items-center"
		:class="onclick ? 'smart-clickable:allow-pointer-events hover:underline cursor-pointer' : ''"
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
}>()
</script>
