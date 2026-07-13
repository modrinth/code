<template>
	<NewModal ref="modal" :header="formatMessage(messages.installToPlay)" :closable="true" no-padding>
		<div v-if="preview" class="flex flex-col gap-6 max-w-[500px] p-6 pb-2">
			<Admonition type="warning" :header="formatMessage(messages.trustWarningHeader)">
				{{ formatMessage(messages.trustWarningDescription) }}
			</Admonition>
			<SharedInstanceInstallSummary :preview="preview" @view-contents="openViewContents" />
			<div v-if="preview.externalFileCount" class="flex items-center gap-2 text-orange">
				<IssuesIcon class="h-5 w-5 flex-none" />
				<span>{{
					formatMessage(messages.externalFileWarning, { count: preview.externalFileCount })
				}}</span>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled
					><button @click="hide">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</button></ButtonStyled
				>
				<ButtonStyled color="brand"
					><button @click="accept">
						<DownloadIcon />{{ formatMessage(messages.installButton) }}
					</button></ButtonStyled
				>
			</div>
		</template>
	</NewModal>
	<ModpackContentModal
		ref="contentModal"
		:header="formatMessage(messages.sharedInstanceContent)"
		:modpack-name="preview?.name ?? ''"
		:modpack-icon-url="preview?.iconUrl ?? undefined"
	/>
</template>

<script setup lang="ts">
import { DownloadIcon, IssuesIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	ModpackContentModal,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads'
import type { SharedInstanceInstallPreview } from '@/helpers/install'

import SharedInstanceInstallSummary from './shared-instance-install-summary.vue'
import { useSharedInstancePreviewContent } from './use-shared-instance-preview-content'

const modal = ref<InstanceType<typeof NewModal>>()
const contentModal = ref<InstanceType<typeof ModpackContentModal>>()
const preview = ref<SharedInstanceInstallPreview | null>(null)
const install = ref<() => void | Promise<void>>(() => {})
const { formatMessage } = useVIntl()
const { load } = useSharedInstancePreviewContent()

async function accept() {
	hide()
	try {
		await install.value()
	} catch (error) {
		console.error('Failed to install shared instance:', error)
	}
}
async function openViewContents() {
	if (!preview.value) return
	contentModal.value?.showLoading()
	try {
		contentModal.value?.show(await load(preview.value))
	} catch (error) {
		console.error('Failed to load shared instance contents:', error)
		contentModal.value?.show([])
	}
}
function show(
	previewValue: SharedInstanceInstallPreview,
	installValue: () => void | Promise<void>,
	event?: MouseEvent,
) {
	preview.value = previewValue
	install.value = installValue
	hide_ads_window()
	modal.value?.show(event)
}
function hide() {
	modal.value?.hide()
	show_ads_window()
}

const messages = defineMessages({
	installToPlay: { id: 'app.modal.install-to-play.header', defaultMessage: 'Install to play' },
	sharedInstanceContent: {
		id: 'app.modal.install-to-play.shared-instance-content',
		defaultMessage: 'Shared instance content',
	},
	trustWarningHeader: {
		id: 'app.modal.install-to-play.trust-warning-header',
		defaultMessage: 'Do you trust this user?',
	},
	trustWarningDescription: {
		id: 'app.modal.install-to-play.trust-warning-description',
		defaultMessage:
			'A shared instance will install files on your computer and may include content not from Modrinth.',
	},
	externalFileWarning: {
		id: 'app.modal.install-to-play.external-file-warning',
		defaultMessage:
			'{count, plural, one {This instance includes # file that is not from Modrinth.} other {This instance includes # files that are not from Modrinth.}}',
	},
	installButton: { id: 'app.modal.install-to-play.install-button', defaultMessage: 'Install' },
})

defineExpose({ show, hide })
</script>
