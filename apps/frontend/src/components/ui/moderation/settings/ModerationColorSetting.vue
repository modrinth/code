<script setup lang="ts">
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import { ColorPicker, defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { computed, onMounted, ref, watch } from 'vue'

import { useTheme } from '~/composables/nuxt-accessors'

const props = defineProps<{
	label: string
	defaultColor: string
	defaultLabel: string
	resetValue: string
}>()
const model = defineModel<string>({ required: true })
const { formatMessage } = useVIntl()
const messages = defineMessages({
	reset: {
		id: 'moderation.settings.reset',
		defaultMessage: 'Reset to default',
	},
})
const theme = useTheme()
const swatch = ref<HTMLElement | null>(null)
const defaultHex = ref('#ffa347')
const pickerColor = computed({
	get: () => model.value || defaultHex.value,
	set: (value: string) => {
		model.value = value
	},
})

function updateDefaultColor() {
	const element = swatch.value
	const view = element?.ownerDocument.defaultView
	if (!element || !view) return
	const channels = view.getComputedStyle(element).color.match(/\d+/g)
	if (channels && channels.length >= 3) {
		defaultHex.value = `#${channels
			.slice(0, 3)
			.map((channel) => Number(channel).toString(16).padStart(2, '0'))
			.join('')}`
	}
}

onMounted(updateDefaultColor)
watch([() => theme.active, () => props.defaultColor], updateDefaultColor, { flush: 'post' })
</script>

<template>
	<div class="flex flex-wrap items-center gap-2">
		<span ref="swatch" class="hidden" :style="{ color: defaultColor }" aria-hidden="true" />
		<span class="text-sm text-secondary">{{ model || defaultLabel }}</span>
		<ColorPicker v-model="pickerColor" :label="label" />
		<IconButton
			v-tooltip="formatMessage(messages.reset)"
			type="quiet"
			size="sm"
			:circular="false"
			:label="formatMessage(messages.reset)"
			:disabled="model === resetValue"
			@click="model = resetValue"
		>
			<RotateCounterClockwiseIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>
