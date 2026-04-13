<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" :closable="true" no-padding>
		<div class="max-w-[500px]">
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
					class="grid grid-cols-[auto_auto_1fr] items-center min-h-10 h-10 gap-2"
				>
					<div class="flex flex-col items-center justify-between">
						<div class="w-[1px] h-2"></div>
						<MinusIcon class="text-red" />
						<div
							:class="index === files.length - 1 ? 'bg-transparent' : 'bg-surface-5'"
							class="w-[1px] h-2 relative top-1"
						></div>
					</div>
					<span class="text-sm shrink-0 whitespace-nowrap">{{
						formatMessage(messages.overwrittenLabel)
					}}</span>
					<span
						v-tooltip="file"
						class="text-sm text-contrast font-medium whitespace-nowrap overflow-hidden text-ellipsis"
					>
						{{ file }}
					</span>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2 pt-4">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="handleProceed">
						<CheckIcon />
						{{ formatMessage(messages.overwriteButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, MinusIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
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
	overwrittenLabel: {
		id: 'files.conflict-modal.overwritten-label',
		defaultMessage: 'Overwritten',
	},
	overwriteButton: {
		id: 'files.conflict-modal.overwrite-button',
		defaultMessage: 'Overwrite',
	},
})

const path = ref('')
const files = ref<string[]>([])

const emit = defineEmits<{
	proceed: [path: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const hasMany = computed(() => files.value.length > 100)

const show = (zipPath: string, conflictingFiles: string[]) => {
	path.value = zipPath
	files.value = conflictingFiles
	modal.value?.show()
}

const hide = () => {
	modal.value?.hide()
}

const handleProceed = () => {
	hide()
	emit('proceed', path.value)
}

defineExpose({ show })
</script>
