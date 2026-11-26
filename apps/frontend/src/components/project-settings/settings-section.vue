<script setup lang="ts">
defineProps<{
	title: string
	titleId?: string
	subtitle?: string
	danger?: boolean
	heading?: 'h2' | 'h3'
}>()
</script>

<template>
	<component
		:is="$slots.default ? 'section' : 'header'"
		:aria-labelledby="$slots.default ? titleId : undefined"
		:aria-label="$slots.default && !titleId ? title : undefined"
	>
		<div :class="{ 'mb-3': $slots.default }">
			<div :class="{ 'flex flex-wrap items-center justify-between gap-3': $slots.actions }">
				<component
					:is="heading ?? 'h2'"
					:id="titleId"
					class="m-0 font-semibold"
					:class="[
						heading === 'h3' ? 'text-lg' : 'settings-section-title text-xl',
						danger ? '!text-red' : 'text-contrast',
					]"
				>
					{{ title }}
				</component>
				<slot name="actions" />
			</div>
			<p v-if="subtitle || $slots.subtitle" class="mb-0 mt-2 leading-tight text-secondary">
				<slot name="subtitle">{{ subtitle }}</slot>
			</p>
		</div>
		<slot />
	</component>
</template>
