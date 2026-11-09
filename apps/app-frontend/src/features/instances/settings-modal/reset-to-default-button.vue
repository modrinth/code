<script setup lang="ts">
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'

defineProps<{ disabled: boolean; setting: string }>()
const emit = defineEmits<{ reset: [] }>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	tooltip: {
		id: 'instance.settings.reset-to-default',
		defaultMessage: 'Reset to default',
	},
	usingDefaults: {
		id: 'instance.settings.using-defaults',
		defaultMessage: 'Using defaults',
	},
	label: {
		id: 'instance.settings.reset-setting-to-default',
		defaultMessage: 'Reset {setting} to default',
	},
})
</script>

<template>
	<span
		v-tooltip="formatMessage(disabled ? messages.usingDefaults : messages.tooltip)"
		class="flex shrink-0"
	>
		<IconButton
			type="outlined"
			size="sm"
			:disabled="disabled"
			:label="formatMessage(messages.label, { setting })"
			@click="emit('reset')"
		>
			<RotateCounterClockwiseIcon aria-hidden="true" />
		</IconButton>
	</span>
</template>
