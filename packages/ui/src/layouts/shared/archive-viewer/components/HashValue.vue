<template>
	<button
		v-tooltip="hash"
		type="button"
		class="inline-flex max-w-full items-center gap-1.5 rounded-lg border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 font-mono text-xs text-primary transition-[filter,transform] hover:brightness-125 active:scale-95 motion-reduce:transition-none"
		:aria-label="formatMessage(messages.copy, { hash })"
		@click="copy"
	>
		<span class="truncate">{{ shortened }}</span>
		<CheckIcon v-if="copied" class="size-3 shrink-0 text-green" aria-hidden="true" />
		<CopyIcon v-else class="size-3 shrink-0" aria-hidden="true" />
	</button>
</template>

<script setup lang="ts">
import { CheckIcon, CopyIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	hash: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	copy: {
		id: 'archive-viewer.hash.copy',
		defaultMessage: 'Copy {hash}',
	},
})

/** Enough of each end to tell hashes apart at a glance; the tooltip shows all of it. */
const EDGE_LENGTH = 8

const shortened = computed(() =>
	props.hash.length > EDGE_LENGTH * 2 + 1
		? `${props.hash.slice(0, EDGE_LENGTH)}…${props.hash.slice(-EDGE_LENGTH)}`
		: props.hash,
)

const copied = ref(false)
let resetTimeout: ReturnType<typeof setTimeout> | undefined

async function copy() {
	await navigator.clipboard.writeText(props.hash)
	copied.value = true
	clearTimeout(resetTimeout)
	resetTimeout = setTimeout(() => (copied.value = false), 2000)
}

onBeforeUnmount(() => clearTimeout(resetTimeout))
</script>
