<template>
	<div
		class="relative flex size-16 shrink-0 overflow-hidden rounded-2xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
	>
		<GeneratedIcon v-if="config" :config="config" alt="Server Icon" class="size-full" />
		<template v-else-if="hasMounted">
			<img
				v-if="cachedImage"
				class="h-full w-full select-none object-fill"
				alt="Server Icon"
				:src="cachedImage"
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

import GeneratedIcon from '#ui/components/base/icon-editor-modal/GeneratedIcon.vue'
import type { IconConfig } from '#ui/components/base/icon-editor-modal/types'
import { useCachedIcon } from '#ui/composables/use-cached-icon'

const hasMounted = ref(false)

onMounted(() => {
	hasMounted.value = true
})

const props = defineProps<{
	image: string | undefined
	config?: IconConfig | null
	disabled?: boolean
}>()

const { src: cachedImage } = useCachedIcon(() => props.image)
</script>
