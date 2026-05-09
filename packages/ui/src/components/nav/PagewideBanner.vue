<template>
	<div
		:class="[
			'banner-grid relative border-b-2 border-solid border-0 z-10',
			containerClasses[variant],
			{ 'no-actions': !$slots.actions, slim: slim },
		]"
	>
		<div
			:class="[
				'grid-area-[title] flex items-center gap-2 font-bold text-[var(--font-size-md)]',
				iconClasses[variant],
			]"
		>
			<component :is="getSeverityIcon(variant)" aria-hidden="true" class="w-6 h-6 flex-shrink-0" />
			<slot name="title" />
		</div>

		<div class="grid-area-[description] flex flex-col gap-[var(--gap-md)]">
			<slot name="description" />
		</div>

		<div v-if="$slots.actions" class="grid-area-[actions] flex items-center gap-2">
			<slot name="actions" />
		</div>

		<div
			v-if="$slots.actions_right || $slots.actions_top_right"
			class="grid-area-[actions_right] flex flex-col gap-2 items-end"
		>
			<div v-if="$slots.actions_top_right" class="flex items-center gap-2 justify-end">
				<slot name="actions_top_right" />
			</div>
			<div v-if="$slots.actions_right" class="flex items-center gap-2 justify-end my-auto">
				<slot name="actions_right" />
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { getSeverityIcon } from '../../utils'

withDefaults(
	defineProps<{
		variant: 'error' | 'warning' | 'info'
		slim?: boolean
	}>(),
	{
		slim: false,
	},
)

const containerClasses = {
	error: 'bg-banners-error-bg text-banners-error-text border-banners-error-border',
	warning: 'bg-banners-warning-bg text-banners-warning-text border-banners-warning-border',
	info: 'bg-banners-info-bg text-banners-info-text border-banners-info-border',
}

const iconClasses = {
	error: 'text-brand-red',
	warning: 'text-brand-orange',
	info: 'text-brand-blue',
}
</script>

<style scoped>
.banner-grid {
	display: grid;
	gap: 0.5rem;
	grid-template-areas:
		'title         actions_right'
		'description   actions_right'
		'actions       actions_right';
	padding-block: var(--gap-xl);
	padding-inline: max(calc((100% - 80rem) / 2 + var(--gap-md)), var(--gap-xl));
}

.banner-grid.no-actions {
	grid-template-areas:
		'title         actions_right'
		'description   actions_right';
}

.banner-grid.slim {
	@apply flex py-4 gap-2 items-center;
}

.grid-area-\[title\] {
	grid-area: title;
}
.grid-area-\[description\] {
	grid-area: description;
}
.grid-area-\[actions\] {
	grid-area: actions;
}
.grid-area-\[actions_right\] {
	grid-area: actions_right;
}

.banner-grid a {
	@apply underline text-current;
}
</style>
