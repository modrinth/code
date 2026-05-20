<template>
	<button
		class="!m-0 inline-flex w-fit select-text items-center gap-2 rounded-[10px] bg-[var(--color-button-bg)] px-2 py-1 font-mono text-sm text-primary transition-[opacity,filter,transform,outline] duration-200 ease-in-out hover:brightness-[1.25] active:scale-95 active:brightness-[0.8] motion-reduce:transition-none [&>svg]:h-[1em] [&>svg]:w-[1em]"
		:title="formatMessage(copiedMessage)"
		@click="copyText"
	>
		<span>{{ text }}</span>
		<CheckIcon v-if="copied" />
		<ClipboardCopyIcon v-else />
	</button>
</template>

<script setup lang="ts">
import { CheckIcon, ClipboardCopyIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessage, useVIntl } from '../../composables/i18n'

const copiedMessage = defineMessage({
	id: 'omorphia.component.copy.action.copy',
	defaultMessage: 'Copy code to clipboard',
})
const { formatMessage } = useVIntl()

const props = defineProps<{ text: string }>()

const copied = ref(false)

async function copyText() {
	await navigator.clipboard.writeText(props.text)
	copied.value = true
}
</script>
