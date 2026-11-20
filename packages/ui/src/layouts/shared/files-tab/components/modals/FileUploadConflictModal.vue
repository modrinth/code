<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		:closable="true"
		no-padding
		:on-hide="handleHide"
	>
		<div v-if="checking" class="flex w-[500px] max-w-full items-center gap-2 p-4 text-secondary">
			<SpinnerIcon class="animate-spin" />
			{{ formatMessage(messages.checkingConflicts) }}
		</div>
		<div v-else class="max-w-[500px]">
			<div class="flex flex-col gap-4 p-4">
				<Admonition type="warning" :header="formatMessage(messages.warningHeader)">
					<span>
						<template v-if="hasMany">
							{{ formatMessage(messages.overwriteManyWarning) }}
						</template>
						<template v-else>
							{{ formatMessage(messages.overwriteWarning, { count: files.length }) }}
						</template>
					</span>
				</Admonition>

				<div v-if="files.length" class="flex gap-2">
					<div class="flex items-center gap-1">
						<MinusIcon />
						{{ formatMessage(messages.overwrittenCount, { count: files.length }) }}
					</div>
				</div>
			</div>

			<div
				v-if="files.length"
				class="flex flex-col bg-surface-2 p-4 max-h-[272px] overflow-y-auto border-t border-b border-r-0 border-l-0 border-solid border-surface-5"
			>
				<div
					v-for="(file, index) in files"
					:key="file"
					class="grid grid-cols-[auto_minmax(0,1fr)] items-center min-h-10 h-10 gap-2"
				>
					<div class="flex flex-col items-center justify-between">
						<div class="w-[1px] h-2"></div>
						<MinusIcon class="text-red" />
						<div
							:class="index === files.length - 1 ? 'bg-transparent' : 'bg-surface-5'"
							class="w-[1px] h-2 relative top-1"
						></div>
					</div>
					<span class="flex min-w-0 text-sm whitespace-nowrap overflow-hidden">
						<IntlFormatted :message-id="messages.overwriteFileLabel" :values="{ path: file }">
							<template #file-path="{ children }">
								<span
									v-tooltip="file"
									class="min-w-0 text-contrast font-medium whitespace-nowrap overflow-hidden text-ellipsis"
								>
									<component :is="() => children" />
								</span>
							</template>
						</IntlFormatted>
					</span>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2 pt-4">
				<Button type="outlined" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="checking" @click="handleProceed">
					<CheckIcon />
					{{ formatMessage(messages.overwriteButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, MinusIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'files.conflict-modal.header',
		defaultMessage: 'Extract summary',
	},
	warningHeader: {
		id: 'files.conflict-modal.warning-header',
		defaultMessage: 'Files will be overwritten',
	},
	overwriteManyWarning: {
		id: 'files.conflict-modal.overwrite-many-warning',
		defaultMessage:
			'Over 100 files will be overwritten if you proceed with extraction; here are some of them.',
	},
	overwriteWarning: {
		id: 'files.conflict-modal.overwrite-warning',
		defaultMessage:
			'The following {count} files already exist on your server, and will be overwritten if you proceed with extraction.',
	},
	overwrittenCount: {
		id: 'files.conflict-modal.overwritten-count',
		defaultMessage: '{count} overwritten',
	},
	overwriteFileLabel: {
		id: 'files.conflict-modal.overwrite-file-label',
		defaultMessage: 'Will overwrite <file-path>{path}</file-path>',
	},
	overwriteButton: {
		id: 'files.conflict-modal.overwrite-button',
		defaultMessage: 'Overwrite',
	},
	checkingConflicts: {
		id: 'files.conflict-modal.checking-conflicts',
		defaultMessage: 'Checking for conflicting files…',
	},
})

const path = ref('')
const files = ref<string[]>([])

const emit = defineEmits<{
	proceed: [path: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const hasMany = computed(() => files.value.length > 100)

const LOADING_DELAY_MS = 1000

const checking = ref(false)
const shown = ref(false)
let checkToken = 0
let pendingToken: number | null = null
let loadingTimer: ReturnType<typeof setTimeout> | undefined

const open = () => {
	clearTimeout(loadingTimer)
	if (shown.value) return
	shown.value = true
	modal.value?.show()
}

/**
 * Starts a conflict check, showing the modal in a loading state if it takes longer than a second.
 * Returns a token for `isCheckPending`, which goes stale if the modal is closed or another check starts.
 */
const startCheck = (zipPath: string): number => {
	path.value = zipPath
	files.value = []
	checking.value = true
	clearTimeout(loadingTimer)
	loadingTimer = setTimeout(open, LOADING_DELAY_MS)
	pendingToken = ++checkToken
	return pendingToken
}

const isCheckPending = (token: number) => token === pendingToken

const showConflicts = (conflictingFiles: string[]) => {
	pendingToken = null
	files.value = conflictingFiles
	checking.value = false
	open()
}

const hide = () => {
	clearTimeout(loadingTimer)
	if (shown.value) {
		modal.value?.hide()
	} else {
		handleHide()
	}
}

const handleHide = () => {
	shown.value = false
	pendingToken = null
}

onBeforeUnmount(() => clearTimeout(loadingTimer))

const handleProceed = () => {
	const proceedPath = path.value
	hide()
	emit('proceed', proceedPath)
}

defineExpose({ startCheck, isCheckPending, showConflicts, hide })
</script>
