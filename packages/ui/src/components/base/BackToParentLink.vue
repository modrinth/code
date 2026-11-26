<template>
	<AutoLink :to="to" class="flex w-fit items-center gap-1 rounded-lg px-2 py-0.5 pl-0 text-link">
		<ChevronLeftIcon aria-hidden="true" class="shrink-0" />
		<slot />
	</AutoLink>
</template>

<script setup lang="ts">
import { ChevronLeftIcon } from '@modrinth/assets'

import AutoLink from './AutoLink.vue'

defineProps<{
	to: unknown
}>()
</script>
