<template>
	<div
		class="flex flex-wrap items-center gap-2 transition-opacity duration-200 motion-reduce:transition-none"
		:class="{ 'pointer-events-none opacity-0': fadeOut || entering }"
	>
		<slot />
	</div>
</template>

<script setup lang="ts">
import { useRouteEnterTransition } from '#ui/composables/route-transitions'

const props = defineProps<{
	fadeIn?: boolean
	fadeOut?: boolean
}>()

const entering = useRouteEnterTransition(props.fadeIn)
</script>
