<template>
	<div
		class="flex flex-wrap items-center gap-2 transition-opacity duration-200 motion-reduce:transition-none"
		:class="{ 'pointer-events-none opacity-0 md:absolute md:right-0': hidden }"
	>
		<slot />
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useRouteEnterTransition } from '#ui/composables/route-transitions'

import { injectPageHeaderContext } from './context'

const props = defineProps<{
	fadeIn?: boolean
	fadeOut?: boolean
}>()

const header = injectPageHeaderContext(null)
const entering = useRouteEnterTransition(props.fadeIn)
const hidden = computed(
	() => props.fadeOut || entering.value || (props.fadeIn && (header?.fadeItemCount.value ?? 0) > 0),
)
</script>
