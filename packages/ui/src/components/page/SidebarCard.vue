<script setup lang="ts">
import { injectPageContext } from '../../providers'

const { hierarchicalSidebarAvailable } = injectPageContext()

defineProps<{
	title: string
}>()
</script>
<template>
	<div
		class="flex flex-col gap-3"
		:class="{
			'mb-4 last:mb-0': !hierarchicalSidebarAvailable,
		}"
	>
		<span class="text-lg font-bold">{{ title }}</span>
		<slot />
	</div>
</template>
