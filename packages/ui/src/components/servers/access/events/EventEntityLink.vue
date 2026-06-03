<template>
	<AutoLink
		:to="entity.to"
		class="inline-flex min-w-0 max-w-full flex-wrap items-center gap-0 @[800px]:flex-nowrap"
		:class="[
			stackSecondary ? '!grid grid-cols-[auto_minmax(0,1fr)] items-start gap-x-2 gap-y-0.5' : '',
			entity.to
				? 'font-medium text-contrast hover:underline'
				: entity.muted
					? 'text-secondary'
					: 'font-medium text-contrast',
			entity.mono ? 'font-mono text-[0.925em]' : '',
			'align-middle',
		]"
	>
		<span
			v-if="entity.iconUrl || entity.icon"
			class="inline-flex shrink-0 items-center justify-center"
			:class="[
				stackSecondary ? 'row-span-2 self-center' : 'mr-1',
				entity.icon ? 'size-7 rounded-lg border border-solid border-surface-5 bg-surface-4 text-secondary' : '',
			]"
		>
			<Avatar
				v-if="entity.iconUrl"
				:src="entity.iconUrl"
				:alt="entity.label"
				size="1.75rem"
				no-shadow
				raised
				:circle="entity.iconShape === 'circle'"
				class="inline-flex shrink-0 border border-solid border-surface-5"
				:class="entity.iconShape === 'circle' ? '!rounded-full' : '!rounded-lg'"
			/>
			<component :is="entity.icon" v-else class="size-4" />
		</span>
		<span
			ref="labelRef"
			v-tooltip="truncatedTooltip(labelRef, entity.title ?? entity.label)"
			class="min-w-0 whitespace-normal break-words leading-7 @[800px]:truncate @[800px]:whitespace-nowrap"
			:class="stackSecondary ? 'leading-6 @[800px]:whitespace-normal @[800px]:break-words' : ''"
		>
			{{ entity.label }}
		</span>
		<span
			v-if="entity.secondaryLabel"
			ref="secondaryLabelRef"
			v-tooltip="truncatedTooltip(secondaryLabelRef, entity.secondaryLabel)"
			class="min-w-0 whitespace-normal break-words text-secondary @[800px]:truncate @[800px]:whitespace-nowrap"
			:class="
				stackSecondary
					? 'col-start-2 leading-5 @[800px]:whitespace-normal @[800px]:break-words'
					: 'entity-secondary-label'
			"
		>
			<template v-if="stackSecondary">{{ entity.secondaryLabel }}</template>
			<template v-else>&nbsp;{{ entity.secondaryLabel }}</template>
		</span>
	</AutoLink>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { truncatedTooltip } from '#ui/utils/truncate'

import type { EventEntity } from './types'

withDefaults(
	defineProps<{
		entity: EventEntity
		stackSecondary?: boolean
	}>(),
	{
		stackSecondary: false,
	},
)

const labelRef = ref<HTMLElement | null>(null)
const secondaryLabelRef = ref<HTMLElement | null>(null)
</script>
