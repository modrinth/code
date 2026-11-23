<script setup lang="ts">
defineProps<{
	title: string
	titleId?: string
	danger?: boolean
}>()
</script>

<template>
	<section :aria-labelledby="titleId" :aria-label="titleId ? undefined : title">
		<div v-if="$slots.actions" class="mb-3 flex items-center justify-between gap-3">
			<h2 :id="titleId" class="settings-section-title !m-0 text-xl font-semibold text-contrast">
				{{ title }}
			</h2>
			<slot name="actions" />
		</div>
		<h2
			v-else
			:id="titleId"
			class="settings-section-title mb-3 mt-0 text-xl font-semibold"
			:class="danger ? '!text-red' : 'text-contrast'"
		>
			{{ title }}
		</h2>
		<slot />
	</section>
</template>
