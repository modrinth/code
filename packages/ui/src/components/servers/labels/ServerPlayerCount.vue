<template>
	<div class="flex gap-1 text-sm font-medium">
		<!-- indicator icon -->

		{{ currentPlayers }} / {{ maxPlayers }} Players
	</div>
</template>

<script setup lang="ts">
interface ServerPlayerCountProps {
	currentPlayers: number
	maxPlayers: number
	online: boolean
}

defineProps<ServerPlayerCountProps>()
</script>
