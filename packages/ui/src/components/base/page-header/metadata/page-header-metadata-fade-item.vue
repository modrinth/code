<template>
	<Transition
		enter-active-class="transition-opacity duration-200 ease-in-out motion-reduce:transition-none"
		enter-from-class="opacity-0"
		leave-active-class="transition-opacity duration-200 ease-in-out motion-reduce:transition-none"
		leave-to-class="opacity-0"
	>
		<PageHeaderMetadataItem v-if="shown">
			<slot />
		</PageHeaderMetadataItem>
	</Transition>
</template>

<script setup lang="ts">
import PageHeaderMetadataItem from './page-header-metadata-item.vue'

defineProps<{
	shown: boolean
}>()
</script>
