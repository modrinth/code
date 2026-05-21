<template>
	<span
		class="inline-flex max-w-full items-center gap-x-1 align-baseline"
		:class="singleLine ? 'min-w-0 flex-nowrap whitespace-nowrap' : 'flex-wrap gap-y-0.5'"
	>
		<template v-for="(entity, index) in visibleEntities" :key="entity.id">
			<EventEntityLink :entity="entity" :class="singleLine ? 'min-w-0' : ''" />
			<span v-if="index < visibleEntities.length - 1" class="text-secondary">,</span>
		</template>
		<Tooltip
			v-if="hiddenCount > 0"
			theme="dismissable-prompt"
			class="inline-flex shrink-0"
			:triggers="['hover', 'focus']"
			:popper-triggers="['hover', 'focus']"
			popper-class="v-popper--interactive"
			placement="top"
			:delay="{ show: 200, hide: 100 }"
			no-auto-focus
		>
			<button
				type="button"
				class="inline-flex cursor-help rounded-full border border-solid border-surface-5 bg-surface-4 px-1.5 py-0 text-xs font-semibold text-secondary"
				:aria-label="hiddenTooltip"
			>
				{{ formatMessage(messages.hiddenCount, { count: hiddenCount }) }}
			</button>
			<template #popper>
				<div class="flex max-w-[22rem] flex-col gap-2 py-0.5">
					<EventEntityLink
						v-for="entity in hiddenEntities"
						:key="entity.id"
						:entity="tooltipEntity(entity)"
						class="min-w-0"
					/>
				</div>
			</template>
		</Tooltip>
	</span>
</template>

<script setup lang="ts">
import { Tooltip } from 'floating-vue'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../../../composables/i18n'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = withDefaults(
	defineProps<{
		entities: EventEntity[]
		limit?: number
		singleLine?: boolean
	}>(),
	{
		limit: 3,
		singleLine: false,
	},
)

const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
	hiddenCount: {
		id: 'servers.audit-log.event.entity-list.hidden-count',
		defaultMessage: '+{count, number}',
	},
})

const visibleEntities = computed(() => props.entities.slice(0, props.limit))
const hiddenEntities = computed(() => props.entities.slice(props.limit))
const hiddenCount = computed(() => hiddenEntities.value.length)
const hiddenTooltip = computed(() => {
	void locale.value
	return new Intl.ListFormat(locale.value, {
		style: 'long',
		type: 'conjunction',
	}).format(hiddenEntities.value.map((entity) => entity.label))
})

function tooltipEntity(entity: EventEntity): EventEntity {
	return {
		...entity,
		secondaryLabel: undefined,
	}
}
</script>
