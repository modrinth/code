<script setup lang="ts">
import { injectPageContext } from '../../providers'

const { hierarchicalSidebarAvailable } = injectPageContext()

defineProps<{
	title: string
}>()
</script>
<template>
	<div
		class="flex flex-col gap-3 p-4 pt-3"
		:class="{
			'card-shadow mb-4 last:mb-0 rounded-2xl bg-surface-3 border border-solid border-surface-4':
				!hierarchicalSidebarAvailable,
		}"
	>
		<h2 class="text-lg font-semibold m-0">{{ title }}</h2>
		<slot />
	</div>
</template>
