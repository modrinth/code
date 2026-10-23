<template>
	<button
		v-if="copyable"
		v-tooltip="tooltip ?? label"
		type="button"
		class="flex min-w-0 max-w-full cursor-pointer items-center gap-1 rounded-full border border-solid border-surface-4 bg-surface-2 px-2 py-0.5 text-xs font-medium text-primary transition-colors hover:bg-surface-3 hover:text-contrast motion-reduce:transition-none"
		:aria-label="formatMessage(copied ? messages.valueCopied : messages.copyValue, { label })"
		@click="copy"
	>
		<CheckIcon v-if="copied" class="size-3.5 shrink-0 text-green" aria-hidden="true" />
		<component
			:is="icon"
			v-else-if="icon"
			class="size-3.5 shrink-0 text-secondary"
			aria-hidden="true"
		/>
		<span class="truncate" :class="{ 'font-mono': mono }">{{ value }}</span>
	</button>
	<span
		v-else
		v-tooltip="tooltip ?? label"
		class="flex min-w-0 max-w-full items-center gap-1 rounded-full border border-solid border-surface-4 bg-surface-2 px-2 py-0.5 text-xs font-medium text-primary"
	>
		<component :is="icon" v-if="icon" class="size-3.5 shrink-0 text-secondary" aria-hidden="true" />
		<span class="sr-only">{{ label }}</span>
		<span class="truncate" :class="{ 'font-mono': mono }">{{ value }}</span>
	</span>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { type Component, onBeforeUnmount, ref } from 'vue'

import { useVIntl } from '#ui/composables/i18n'

import { sectionMessages as messages } from './messages'

/**
 * A short value shown without a heading (an ID, a date, a count), named by its tooltip, which
 * copies `copyText` (or the value itself) when clicked.
 */
const props = withDefaults(
	defineProps<{
		/** What the value is, shown on hover when there's no `tooltip`. */
		label: string
		value: string
		icon?: Component
		tooltip?: string
		/** Copied in place of the value shown, e.g. a full date for a short one. */
		copyText?: string
		copyable?: boolean
		mono?: boolean
	}>(),
	{ copyable: true },
)

const { formatMessage } = useVIntl()

const copied = ref(false)
let copiedReset: ReturnType<typeof setTimeout> | undefined

async function copy() {
	await navigator.clipboard.writeText(props.copyText ?? props.value)
	copied.value = true
	clearTimeout(copiedReset)
	copiedReset = setTimeout(() => (copied.value = false), 2000)
}

onBeforeUnmount(() => clearTimeout(copiedReset))
</script>
