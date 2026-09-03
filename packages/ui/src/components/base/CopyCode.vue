<template>
	<button
		class="rounded-lg border border-solid border-surface-5 bg-surface-2 text-xs !m-0 inline-flex w-fit select-text items-center gap-2 px-2 py-1 font-mono text-primary transition-[opacity,filter,transform,outline] duration-200 ease-in-out hover:brightness-[1.25] active:scale-95 motion-reduce:transition-none [&>svg]:h-[1em] [&>svg]:w-[1em]"
		:title="formatMessage(copiedMessage)"
		@click="copyText"
	>
		<span>{{ displayText ?? text }}</span>
		<CheckIcon v-if="copied" />
		<CopyIcon v-else />
	</button>
</template>

<script setup lang="ts">
import { CheckIcon, CopyIcon } from '@modrinth/assets'
import { onBeforeUnmount, ref } from 'vue'

import { defineMessage, useVIntl } from '../../composables/i18n'

const copiedMessage = defineMessage({
	id: 'omorphia.component.copy.action.copy',
	defaultMessage: 'Copy code to clipboard',
})
const { formatMessage } = useVIntl()

const props = defineProps<{
	text: string
	displayText?: string
}>()

const copied = ref(false)
let copiedResetTimeout: ReturnType<typeof setTimeout> | undefined

async function copyText() {
	await navigator.clipboard.writeText(props.text)
	copied.value = true
	clearTimeout(copiedResetTimeout)
	copiedResetTimeout = setTimeout(() => {
		copied.value = false
	}, 2000)
}

onBeforeUnmount(() => clearTimeout(copiedResetTimeout))
</script>
