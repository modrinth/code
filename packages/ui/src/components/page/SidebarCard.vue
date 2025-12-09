<script setup lang="ts">
import { injectPageContext } from '../../providers'

const { hierarchicalSidebarAvailable } = injectPageContext()

defineProps<{
	title: string
}>()
</script>
<template>
	<div
		class="flex flex-col gap-3 p-4"
		:class="{
			'card-shadow mb-4 last:mb-0 rounded-2xl bg-bg-raised': !hierarchicalSidebarAvailable,
		}"
	>
		<span class="font-semibold">{{ title }}</span>
		<slot />
	</div>
</template>
