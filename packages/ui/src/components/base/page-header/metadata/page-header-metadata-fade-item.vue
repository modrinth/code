<template>
	<Transition
		enter-active-class="transition-opacity duration-200 ease-in-out motion-reduce:transition-none"
		enter-from-class="opacity-0"
		leave-active-class="transition-opacity duration-200 ease-in-out motion-reduce:transition-none"
		leave-to-class="opacity-0"
		@after-leave="present = false"
	>
		<PageHeaderMetadataItem v-if="shown">
			<slot />
		</PageHeaderMetadataItem>
	</Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

import { injectPageHeaderContext } from '../context'
import PageHeaderMetadataItem from './page-header-metadata-item.vue'

const props = defineProps<{
	shown: boolean
}>()

const header = injectPageHeaderContext(null)
const present = ref(props.shown)

watch(
	() => props.shown,
	(shown) => {
		if (shown) present.value = true
	},
)

if (header) {
	watch(
		present,
		(isPresent, _, onCleanup) => {
			if (!isPresent) return
			header.fadeItemCount.value++
			onCleanup(() => header.fadeItemCount.value--)
		},
		{ immediate: true },
	)
}
</script>
