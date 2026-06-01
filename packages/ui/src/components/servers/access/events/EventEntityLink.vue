<template>
	<AutoLink
		:to="entity.to"
		class="inline-flex min-w-0 max-w-full flex-wrap items-center gap-1 @[800px]:flex-nowrap"
		:class="[
			entity.to
				? 'font-medium text-contrast hover:underline'
				: entity.muted
					? 'text-secondary'
					: 'font-medium text-contrast',
			entity.mono ? 'font-mono text-[0.925em]' : '',
			'align-middle',
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
			class="ml-1 inline-flex shrink-0 border border-solid border-surface-5"
			:class="entity.iconShape === 'circle' ? '!rounded-full' : '!rounded-lg'"
		/>
		<span
			v-else-if="entity.icon"
			class="ml-1 inline-flex size-7 shrink-0 items-center justify-center rounded-lg border border-solid border-surface-5 bg-surface-4 text-secondary"
		>
			<component :is="entity.icon" class="size-4" />
		</span>
		<span
			ref="labelRef"
			v-tooltip="truncatedTooltip(labelRef, entity.title ?? entity.label)"
			class="min-w-0 whitespace-normal break-words leading-7 @[800px]:truncate @[800px]:whitespace-nowrap"
		>
			{{ entity.label }}
		</span>
		<span
			v-if="entity.secondaryLabel"
			ref="secondaryLabelRef"
			v-tooltip="truncatedTooltip(secondaryLabelRef, entity.secondaryLabel)"
			class="min-w-0 whitespace-normal break-words text-secondary @[800px]:truncate @[800px]:whitespace-nowrap"
		>
			{{ entity.secondaryLabel }}
		</span>
	</AutoLink>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { truncatedTooltip } from '#ui/utils/truncate'

import type { EventEntity } from './types'

defineProps<{
	entity: EventEntity
}>()

const labelRef = ref<HTMLElement | null>(null)
const secondaryLabelRef = ref<HTMLElement | null>(null)
</script>
