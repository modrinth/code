<template>
	<Admonition
		:type="op.state === 'done' ? 'success' : op.state?.startsWith('fail') ? 'critical' : 'info'"
		:dismissible="dismissible && isTerminal"
		:progress="'progress' in op ? (op.progress ?? 0) : 0"
		:progress-color="op.state === 'done' ? 'green' : op.state?.startsWith('fail') ? 'red' : 'blue'"
		:waiting="op.state === 'queued' || !op.progress || op.progress === 0"
		@dismiss="$emit('dismiss')"
	>
		<template #icon="{ iconClass }">
			<PackageOpenIcon :class="iconClass" />
		</template>
		<template #header>{{ title }}</template>
		<span class="text-secondary">
			<span>
				{{
					formatMessage(messages.extracted, {
						size: formatBytes(op.bytes_processed ?? 0),
					})
				}}
			</span>
			<span v-if="'current_file' in op && op.current_file">
				. {{ formatMessage(messages.currentFile, { file: op.current_file?.split('/')?.pop() }) }}
			</span>
		</span>
		<template v-if="op.id" #top-right-actions>
			<ButtonStyled v-if="!isTerminal" type="outlined" color="blue">
				<button class="!border" type="button" @click="ctx.dismissOperation(op.id!, 'cancel')">
					{{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { PackageOpenIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { useFormatBytes } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { FileOperation } from '#ui/layouts/shared/files-tab/types'
import { injectModrinthServerContext } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

defineEmits<{ dismiss: [] }>()

const props = defineProps<{
	op: FileOperation
	dismissible: boolean
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const ctx = injectModrinthServerContext()

const messages = defineMessages({
	extracting: {
		id: 'files.operations.extracting',
		defaultMessage: 'Extracting {source}',
	},
	extractingCompleted: {
		id: 'files.operations.extracting-completed',
		defaultMessage: 'Extracting {source} finished',
	},
	extractingFailed: {
		id: 'files.operations.extracting-failed',
		defaultMessage: 'Extracting {source} failed',
	},
	modpackFromUrl: {
		id: 'files.operations.modpack-from-url',
		defaultMessage: 'modpack from URL',
	},
	extracted: {
		id: 'files.operations.extracted',
		defaultMessage: '{size} extracted',
	},
	currentFile: {
		id: 'files.operations.current-file',
		defaultMessage: 'Current file: {file}',
	},
})

const isTerminal = computed(() => props.op.state === 'done' || !!props.op.state?.startsWith('fail'))
const sourceName = computed(() =>
	props.op.src.includes('https://') ? formatMessage(messages.modpackFromUrl) : props.op.src,
)

const title = computed(() => {
	if (props.op.state === 'done') {
		return formatMessage(messages.extractingCompleted, { source: sourceName.value })
	}
	if (props.op.state?.startsWith('fail')) {
		return formatMessage(messages.extractingFailed, { source: sourceName.value })
	}
	return formatMessage(messages.extracting, { source: sourceName.value })
})
</script>
