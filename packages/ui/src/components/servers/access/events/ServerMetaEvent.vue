<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #name>
				<EventEntityLink :entity="nameEntity" />
			</template>
			<template #subdomain>
				<EventEntityLink :entity="subdomainEntity" />
			</template>
			<template #specs>
				<EventInlineText :text="specsLabel" class="align-middle font-medium text-contrast" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import EventInlineText from './EventInlineText.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind: 'name' | 'subdomain' | 'plan'
	name?: string
	subdomain?: string
	newSpecs?: Record<string, unknown>
}>()

const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
	nameChanged: {
		id: 'servers.audit-log.event.server-name-changed',
		defaultMessage: 'Changed server name to <name></name>',
	},
	subdomainChanged: {
		id: 'servers.audit-log.event.server-subdomain-changed',
		defaultMessage: 'Changed server subdomain to <subdomain></subdomain>',
	},
	planChanged: {
		id: 'servers.audit-log.event.server-plan-changed',
		defaultMessage: 'Changed plan to <specs></specs>',
	},
	changed: {
		id: 'servers.audit-log.event.server-metadata-changed',
		defaultMessage: 'Changed server metadata',
	},
	cpuSpec: {
		id: 'servers.audit-log.event.server-plan.cpu',
		defaultMessage: '{count, plural, one {# CPU} other {# CPUs}}',
	},
	ramGb: {
		id: 'servers.audit-log.event.server-plan.ram-gb',
		defaultMessage: '{amount, number} GB RAM',
	},
	ramMb: {
		id: 'servers.audit-log.event.server-plan.ram-mb',
		defaultMessage: '{amount, number} MB RAM',
	},
	storageGb: {
		id: 'servers.audit-log.event.server-plan.storage-gb',
		defaultMessage: '{amount, number} GB storage',
	},
	storageMb: {
		id: 'servers.audit-log.event.server-plan.storage-mb',
		defaultMessage: '{amount, number} MB storage',
	},
	newPlan: {
		id: 'servers.audit-log.event.server-plan.new-plan',
		defaultMessage: 'new plan',
	},
})

const kindMessages: Record<string, MessageDescriptor> = {
	name: messages.nameChanged,
	subdomain: messages.subdomainChanged,
	plan: messages.planChanged,
}

const message = computed(() => kindMessages[props.kind] ?? messages.changed)
const nameEntity = computed<EventEntity>(() => ({
	id: props.name ?? '',
	label: props.name ?? '',
}))
const subdomainEntity = computed<EventEntity>(() => ({
	id: props.subdomain ?? '',
	label: props.subdomain ?? '',
	mono: true,
}))

const specsLabel = computed(() => {
	void locale.value

	const cpu = numberValue(props.newSpecs?.cpu)
	const memory = numberValue(props.newSpecs?.memory_mb)
	const storage = numberValue(props.newSpecs?.storage_mb)
	const parts = []
	if (cpu != null) parts.push(formatMessage(messages.cpuSpec, { count: cpu }))
	if (memory != null) parts.push(formatMemoryMb(memory))
	if (storage != null) parts.push(formatStorageMb(storage))
	return parts.length > 0 ? parts.join(' / ') : formatMessage(messages.newPlan)
})

function numberValue(value: unknown): number | null {
	return typeof value === 'number' && Number.isFinite(value) ? value : null
}

function formatMemoryMb(value: number): string {
	if (value >= 1024) return formatMessage(messages.ramGb, { amount: Math.round(value / 1024) })
	return formatMessage(messages.ramMb, { amount: value })
}

function formatStorageMb(value: number): string {
	if (value >= 1024) {
		return formatMessage(messages.storageGb, { amount: Math.round(value / 1024) })
	}
	return formatMessage(messages.storageMb, { amount: value })
}
</script>
