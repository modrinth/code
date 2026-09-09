<template>
	<IconButton
		v-tooltip="label"
		:label="label"
		:disabled="copied"
		class="relative grid place-items-center overflow-hidden"
		@click="copyToClipboard"
	>
		<CheckIcon
			class="absolute transition-all ease-in-out"
			:class="copied ? 'translate-y-0' : 'translate-y-7'"
		/>
		<LinkIcon
			class="absolute transition-all ease-in-out"
			:class="copied ? '-translate-y-7' : 'translate-y-0'"
		/>
	</IconButton>
</template>

<script setup lang="ts">
import { CheckIcon, LinkIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref } from 'vue'

import { defineMessage, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import IconButton from './buttons/IconButton.vue'

const copyLinkMessage = commonMessages.copyLinkButton
const copiedToClipboardMessage = defineMessage({
	id: 'button.copied-to-clipboard',
	defaultMessage: 'Copied to clipboard',
})

const { formatMessage } = useVIntl()

const props = defineProps<{
	url: string
	copyLabel?: string
	copiedLabel?: string
}>()

const copied = ref(false)
let copiedResetTimeout: ReturnType<typeof setTimeout> | undefined

const label = computed(() => {
	if (copied.value) {
		return props.copiedLabel ?? formatMessage(copiedToClipboardMessage)
	}

	return props.copyLabel ?? formatMessage(copyLinkMessage)
})

async function copyToClipboard() {
	await navigator.clipboard.writeText(props.url)
	copied.value = true
	clearTimeout(copiedResetTimeout)
	copiedResetTimeout = setTimeout(() => {
		copied.value = false
	}, 3000)
}

onBeforeUnmount(() => clearTimeout(copiedResetTimeout))
</script>
