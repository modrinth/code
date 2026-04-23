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
		<template #header>
			{{
				formatMessage(messages.extracting, {
					source: op.src.includes('https://') ? formatMessage(messages.modpackFromUrl) : op.src,
				})
			}}
			<span v-if="op.state === 'done'" class="font-normal text-green">
				— {{ formatMessage(commonMessages.doneLabel) }}</span
			>
			<span v-else-if="op.state?.startsWith('fail')" class="font-normal text-red">
				— {{ formatMessage(messages.failed) }}</span
			>
		</template>
		<span class="text-secondary">
			{{
				formatMessage(messages.extracted, {
					size: 'bytes_processed' in op ? formatBytes(op.bytes_processed ?? 0) : '0 B',
				})
			}}
			<template v-if="'current_file' in op && op.current_file">
				— {{ op.current_file?.split('/')?.pop() }}
			</template>
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
import { formatBytes } from '@modrinth/utils'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
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
const ctx = injectModrinthServerContext()

const isTerminal = computed(() => props.op.state === 'done' || !!props.op.state?.startsWith('fail'))

const messages = defineMessages({
	extracting: {
		id: 'files.operations.extracting',
		defaultMessage: 'Extracting {source}',
	},
	modpackFromUrl: {
		id: 'files.operations.modpack-from-url',
		defaultMessage: 'modpack from URL',
	},
	failed: {
		id: 'files.operations.failed',
		defaultMessage: 'Failed',
	},
	extracted: {
		id: 'files.operations.extracted',
		defaultMessage: '{size} extracted',
	},
})
</script>
