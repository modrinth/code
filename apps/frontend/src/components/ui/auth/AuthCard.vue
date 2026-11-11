<template>
	<div
		class="flex w-full flex-col gap-6 !p-6"
		:class="
			embedded
				? undefined
				: 'universal-card mx-auto max-w-[27rem] border border-solid border-surface-5'
		"
	>
		<slot />
	</div>
</template>

<script setup lang="ts">
import { isAuthEmbedded } from '@/composables/auth-modal.ts'

const embedded = isAuthEmbedded()
</script>
