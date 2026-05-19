<template>
	<span class="inline-flex max-w-full flex-wrap items-center gap-x-1 gap-y-0.5 align-baseline">
		<template v-for="(entity, index) in visibleEntities" :key="entity.id">
			<EventEntityLink :entity="entity" />
			<span v-if="index < visibleEntities.length - 1" class="text-secondary">,</span>
		</template>
		<span
			v-if="hiddenCount > 0"
			v-tooltip="hiddenTooltip"
			class="inline-flex rounded-full border border-solid border-surface-5 bg-surface-4 px-1.5 text-xs font-semibold text-secondary"
		>
			{{ formatMessage(messages.hiddenCount, { count: hiddenCount }) }}
		</span>
	</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../../../composables/i18n'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = withDefaults(
	defineProps<{
		entities: EventEntity[]
		limit?: number
	}>(),
	{
		limit: 3,
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
</script>
