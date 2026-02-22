<template>
	<div class="flex gap-1.5 items-center">
		<Avatar :src="icon" size="24px" />
		<span class="truncate font-medium text-contrast">
			{{ name }}
		</span>
	</div>
</template>

<script setup lang="ts">
import Avatar from '../../base/Avatar.vue'

defineProps<{
	name: string
	icon?: string
}>()
</script>
