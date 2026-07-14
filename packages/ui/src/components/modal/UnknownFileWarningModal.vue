<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		:on-hide="handleHide"
		max-width="544px"
		width="544px"
	>
		<div class="flex flex-col items-end gap-6">
			<Admonition
				type="warning"
				:header="
					formatMessage(
						mode === 'modpack' ? messages.modpackWarningTitle : messages.modWarningTitle,
					)
				"
				class="w-full"
			>
				<span class="font-medium text-contrast">{{ fileName }}</span>
				{{
					formatMessage(mode === 'modpack' ? messages.modpackWarningBody : messages.modWarningBody)
				}}
			</Admonition>

			<p class="m-0 w-full leading-6 text-primary">
				{{ formatMessage(messages.reviewedFiles) }}
			</p>

			<div
				v-if="mode === 'modpack'"
				class="w-full overflow-hidden rounded-2xl border border-solid border-surface-4 shadow-sm"
			>
				<table class="w-full border-collapse text-left">
					<thead class="bg-surface-3 text-contrast">
						<tr>
							<th class="p-3 font-medium">
								{{ formatMessage(messages.unrecognizedFiles) }}
							</th>
						</tr>
					</thead>
					<tbody class="text-primary">
						<tr
							v-for="(externalFile, index) in externalFilesInModpack"
							:key="`${externalFile}-${index}`"
							:class="index % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
						>
							<td class="break-words border-0 border-t border-solid border-surface-4 p-3">
								{{ externalFile }}
							</td>
						</tr>
					</tbody>
				</table>
			</div>

			<p class="m-0 w-full font-medium leading-6 text-orange">
				{{ formatMessage(messages.malwareWarning) }}
			</p>

			<Checkbox
				v-if="mode === 'mod'"
				v-model="dontShowAgain"
				class="w-full"
				:label="formatMessage(messages.dontShowAgain)"
			/>

			<div class="flex items-center gap-2">
				<ButtonStyled type="transparent" color="orange">
					<button type="button" @click="continueInstallation">
						{{ formatMessage(messages.installAnyway) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" @click="cancelInstallation">
						<BanIcon aria-hidden="true" />
						{{ formatMessage(messages.dontInstall) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { BanIcon } from '@modrinth/assets'
import { ref, useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import Admonition from '../base/Admonition.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import NewModal from './NewModal.vue'

withDefaults(
	defineProps<{
		mode: 'modpack' | 'mod'
		fileName: string
		externalFilesInModpack?: string[]
	}>(),
	{
		externalFilesInModpack: () => [],
	},
)

const emit = defineEmits<{
	cancel: []
	continue: [dontShowAgain: boolean]
}>()

const { formatMessage } = useVIntl()
const modal = useTemplateRef('modal')
const dontShowAgain = ref(false)
let closingWithAction = false

const messages = defineMessages({
	header: {
		id: 'unknown-file-warning-modal.header',
		defaultMessage: 'Confirm installation',
	},
	modpackWarningTitle: {
		id: 'unknown-file-warning-modal.modpack-warning-title',
		defaultMessage: 'Unknown files warning',
	},
	modWarningTitle: {
		id: 'unknown-file-warning-modal.mod-warning-title',
		defaultMessage: 'Unknown file warning',
	},
	modpackWarningBody: {
		id: 'unknown-file-warning-modal.modpack-warning-body',
		defaultMessage:
			' contains files that aren’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	modWarningBody: {
		id: 'unknown-file-warning-modal.mod-warning-body',
		defaultMessage:
			' isn’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	reviewedFiles: {
		id: 'unknown-file-warning-modal.reviewed-files',
		defaultMessage:
			'A file is only reviewed if it’s published to Modrinth, regardless of its file format (including .mrpack).',
	},
	unrecognizedFiles: {
		id: 'unknown-file-warning-modal.unrecognized-files',
		defaultMessage: 'Unrecognized files',
	},
	malwareWarning: {
		id: 'unknown-file-warning-modal.malware-warning',
		defaultMessage:
			'Malware is often distributed through mod files by sharing them on platforms like Discord.',
	},
	dontShowAgain: {
		id: 'unknown-file-warning-modal.dont-show-again',
		defaultMessage: 'Don’t show this warning again',
	},
	installAnyway: {
		id: 'unknown-file-warning-modal.install-anyway',
		defaultMessage: 'Install anyway',
	},
	dontInstall: {
		id: 'unknown-file-warning-modal.dont-install',
		defaultMessage: 'Dont install',
	},
})

function show() {
	dontShowAgain.value = false
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function handleHide() {
	dontShowAgain.value = false
	if (closingWithAction) {
		closingWithAction = false
		return
	}
	emit('cancel')
}

function cancelInstallation() {
	closingWithAction = true
	modal.value?.hide()
	emit('cancel')
}

function continueInstallation() {
	const shouldSkipNextTime = dontShowAgain.value
	closingWithAction = true
	modal.value?.hide()
	emit('continue', shouldSkipNextTime)
}

defineExpose({ show, hide })
</script>
