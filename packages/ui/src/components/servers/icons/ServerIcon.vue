<template>
	<div
		class="experimental-styles-within relative flex size-16 shrink-0 overflow-hidden rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
	>
		<client-only>
			<img
				v-if="image"
				class="h-full w-full select-none object-fill"
				alt="Server Icon"
				:src="image"
			/>
			<img
				v-else
				class="h-full w-full select-none object-fill"
				alt="Server Icon"
				:src="MinecraftServerIcon"
			/>
		</client-only>
		<div v-if="disabled" class="absolute inset-0 bg-surface-1 opacity-50" />
	</div>
</template>

<script setup lang="ts">
import { MinecraftServerIcon } from '@modrinth/assets'

defineProps<{
	image: string | undefined
	disabled?: boolean
}>()
</script>
