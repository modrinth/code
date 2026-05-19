<template>
	<AutoLink
		:to="entity.to"
		class="inline-flex max-w-full items-center gap-1 font-semibold"
		:class="[
			entity.to ? 'text-contrast hover:underline' : entity.muted ? 'text-secondary' : 'text-contrast',
			entity.mono ? 'font-mono text-[0.925em]' : '',
			hasIcon ? 'align-[-0.45rem]' : 'align-baseline',
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
		<span
			v-else-if="entity.icon"
			class="inline-flex size-7 shrink-0 items-center justify-center rounded-lg border border-solid border-surface-5 bg-surface-4 text-secondary"
		>
			<component :is="entity.icon" class="size-4" />
		</span>
		<span v-tooltip="entity.title ?? entity.label" class="min-w-0 truncate leading-7">
			{{ entity.label }}
		</span>
		<span v-if="entity.secondaryLabel" class="shrink-0 text-secondary">
			{{ entity.secondaryLabel }}
		</span>
	</AutoLink>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'

import type { EventEntity } from './types'

const props = defineProps<{
	entity: EventEntity
}>()

const hasIcon = computed(() => Boolean(props.entity.iconUrl || props.entity.icon))
</script>
