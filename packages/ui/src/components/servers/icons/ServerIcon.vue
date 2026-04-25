<template>
	<div
		class="experimental-styles-within relative flex size-16 shrink-0 overflow-hidden rounded-2xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
	>
		<template v-if="hasMounted">
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
		</template>
		<img
			v-else
			class="h-full w-full select-none object-fill"
			alt="Server Icon"
			:src="MinecraftServerIcon"
		/>
		<div v-if="disabled" class="absolute inset-0 bg-surface-1 opacity-50" />
	</div>
</template>

<script setup lang="ts">
import { MinecraftServerIcon } from '@modrinth/assets'
import { onMounted, ref } from 'vue'

const hasMounted = ref(false)

onMounted(() => {
	hasMounted.value = true
})

defineProps<{
	image: string | undefined
	disabled?: boolean
}>()
</script>
