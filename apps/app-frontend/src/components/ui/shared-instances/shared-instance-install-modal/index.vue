<template>
	<NewModal
		ref="modal"
		:header="
			reportMode
				? formatMessage(messages.reportSharedInstance)
				: formatMessage(messages.installToPlay)
		"
		:closable="!submitLoading"
		:on-hide="handleHide"
		:max-width="reportMode ? '816px' : '544px'"
		:width="reportMode ? '816px' : '544px'"
		:no-padding="reportMode"
		:scrollable="reportMode"
	>
		<div
			v-if="preview"
			class="flex w-full flex-col gap-6"
			:class="{ 'p-6 pb-2': reportMode }"
		>
			<Admonition v-if="reportMode" type="info">
				<div class="flex flex-col gap-2">
					<p class="m-0">
						<IntlFormatted :message-id="messages.reportDescription">
							<template #rules-link="{ children }">
								<AutoLink class="text-link hover:underline" to="https://modrinth.com/legal/rules">
									<component :is="() => children" />
								</AutoLink>
							</template>
							<template #terms-link="{ children }">
								<AutoLink class="text-link hover:underline" to="https://modrinth.com/legal/terms">
									<component :is="() => children" />
								</AutoLink>
							</template>
						</IntlFormatted>
					</p>
					<ul class="m-0 list-disc pl-5">
						<li>
							<IntlFormatted :message-id="messages.supportAndBugReports">
								<template #support-link="{ children }">
									<AutoLink class="text-link hover:underline" to="https://support.modrinth.com">
										<component :is="() => children" />
									</AutoLink>
								</template>
								<template #github-link="{ children }">
									<AutoLink
										class="text-link hover:underline"
										to="https://github.com/modrinth/code/issues"
									>
										<component :is="() => children" />
									</AutoLink>
								</template>
							</IntlFormatted>
						</li>
						<li>
							<IntlFormatted :message-id="messages.legalClaims">
								<template #copyright-link="{ children }">
									<AutoLink
										class="text-link hover:underline"
										to="https://modrinth.com/legal/copyright"
									>
										<component :is="() => children" />
									</AutoLink>
								</template>
							</IntlFormatted>
						</li>
					</ul>
				</div>
			</Admonition>
			<p v-else class="m-0 text-primary">
				{{ formatMessage(messages.inviteWarning) }}
			</p>
			<SharedInstanceInstallSummary
				:preview="preview"
				:heading="reportMode ? formatMessage(messages.contentYouAreReporting) : undefined"
				@view-contents="openViewContents"
			/>
			<Transition
				enter-active-class="overflow-hidden transition-all duration-200 ease-out"
				enter-from-class="max-h-0 opacity-0"
				enter-to-class="max-h-[400px] opacity-100"
				leave-active-class="overflow-hidden transition-all duration-200 ease-in"
				leave-from-class="max-h-[400px] opacity-100"
				leave-to-class="max-h-0 opacity-0"
			>
				<div v-if="reportMode" class="flex flex-col gap-6">
					<div class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.reportReason) }}
						</span>
						<Combobox v-model="reportReason" :options="reportReasonOptions" />
					</div>
					<div class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.additionalContext) }}
						</span>
						<MarkdownEditor
							v-model="additionalContext"
							:placeholder="formatMessage(messages.additionalContextPlaceholder)"
							:on-image-upload="onImageUpload"
							:min-height="120"
							:max-height="240"
						/>
					</div>
					<div v-if="reportOnly" class="flex flex-col gap-2">
						<Checkbox v-model="deleteInstance" :label="formatMessage(messages.deleteInstance)" />
					</div>
				</div>
			</Transition>
			<Admonition
				v-if="!reportMode && hasExternalFiles"
				type="warning"
				:header="formatMessage(messages.unknownFilesWarning)"
			>
				{{ formatMessage(messages.unknownFilesDescription) }}
			</Admonition>

			<div v-if="!reportMode && hasExternalFiles" class="relative w-full">
				<div
					ref="externalFileTable"
					class="max-h-[242px] overflow-y-auto rounded-2xl"
					@scroll="checkTableScrollState"
				>
					<Table
						:columns="externalFileColumns"
						:data="externalFileRows"
						row-key="id"
						virtualized
						:virtual-row-height="48"
						class="shadow-sm"
					>
						<template #cell-name="{ value }">
							<span class="block truncate" :title="String(value)">{{ value }}</span>
						</template>
					</Table>
				</div>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>

			<p v-if="!reportMode && hasExternalFiles" class="m-0 text-primary">
				{{ formatMessage(messages.reviewedFiles) }}
			</p>
			<div v-if="!reportMode" class="flex w-full items-center justify-between gap-2">
				<ButtonStyled color="red" type="transparent">
					<button @click="reportMode = true">
						<ReportIcon />{{ formatMessage(commonMessages.reportButton) }}
					</button>
				</ButtonStyled>
				<div class="flex items-center gap-2">
					<template v-if="hasExternalFiles">
						<ButtonStyled type="transparent" color="orange">
							<button @click="accept">
								{{ formatMessage(messages.installAnyway) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button @click="handleCancel">
								<BanIcon />{{ formatMessage(messages.dontInstall) }}
							</button>
						</ButtonStyled>
					</template>
					<template v-else>
						<ButtonStyled type="outlined">
							<button class="!border" @click="handleCancel">
								<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button @click="accept">
								<DownloadIcon />{{ formatMessage(messages.installButton) }}
							</button>
						</ButtonStyled>
					</template>
				</div>
			</div>
		</div>
		<template v-if="reportMode" #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border" :disabled="submitLoading" @click="handleCancel">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canSubmitReport" @click="submitReport">
						<SpinnerIcon v-if="submitLoading" class="animate-spin" />
						<SendIcon v-else />
						{{ formatMessage(commonMessages.reportButton) }}
					</button>
				</ButtonStyled>
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
import type { Labrinth } from '@modrinth/api-client'
import { BanIcon, DownloadIcon, ReportIcon, SendIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	AutoLink,
	ButtonStyled,
	Checkbox,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	MarkdownEditor,
	ModpackContentModal,
	NewModal,
	Table,
	type TableColumn,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, ref } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads'
import { toError } from '@/helpers/errors'
import type { SharedInstanceInstallPreview } from '@/helpers/install'
import { create_report } from '@/helpers/reports'

import SharedInstanceInstallSummary from './shared-instance-install-summary.vue'
import { useSharedInstancePreviewContent } from './use-shared-instance-preview-content'

type ExternalFileColumn = 'name'
type ExternalFileRow = {
	id: string
	name: string
}

const modal = ref<InstanceType<typeof NewModal>>()
const contentModal = ref<InstanceType<typeof ModpackContentModal>>()
const externalFileTable = ref<HTMLElement | null>(null)
const preview = ref<SharedInstanceInstallPreview | null>(null)
const install = ref<() => void | Promise<void>>(() => {})
const reportMode = ref(false)
const reportOnly = ref(false)
type ReportReason = 'malicious' | 'inappropriate' | 'spam'
const reportReason = ref<ReportReason>('malicious')
const additionalContext = ref('')
const deleteInstance = ref(true)
const submitLoading = ref(false)
const uploadedImageIDs = ref<string[]>([])
const emit = defineEmits<{
	reported: [deleteInstance: boolean]
}>()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { addNotification, handleError } = injectNotificationManager()
const { load } = useSharedInstancePreviewContent()
const {
	showTopFade: showTableTopFade,
	showBottomFade: showTableBottomFade,
	checkScrollState: checkTableScrollState,
	forceCheck: forceCheckTableScroll,
} = useScrollIndicator(externalFileTable)
const hasExternalFiles = computed(() => Boolean(preview.value?.externalFiles.length))
const externalFileRows = computed<ExternalFileRow[]>(() =>
	(preview.value?.externalFiles ?? [])
		.map((file, index) => ({
			id: `${index}-${file.fileType}-${file.fileName}`,
			name: file.fileName,
		}))
		.sort((left, right) => left.name.localeCompare(right.name)),
)
const reportReasonOptions = computed<ComboboxOption<ReportReason>[]>(() => [
	{ value: 'malicious', label: formatMessage(messages.maliciousReason) },
	{ value: 'inappropriate', label: formatMessage(messages.inappropriateReason) },
	{ value: 'spam', label: formatMessage(messages.spamReason) },
])
const canSubmitReport = computed(
	() => Boolean(preview.value && additionalContext.value.trim()) && !submitLoading.value,
)

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
async function submitReport() {
	const reportPreview = preview.value
	const body = additionalContext.value.trim()
	if (!reportPreview || !body || submitLoading.value) return

	submitLoading.value = true
	try {
		const uploadedImages = uploadedImageIDs.value.slice(-10)
		await create_report({
			report_type: reportReason.value,
			item_type: 'shared-instance',
			item_id: `${reportPreview.sharedInstanceId}/${reportPreview.version}`,
			body,
			uploaded_images: uploadedImages,
		})

		const shouldDeleteInstance = reportOnly.value && deleteInstance.value
		hide()
		addNotification({
			type: 'success',
			title: formatMessage(messages.reportSubmitted),
		})
		emit('reported', shouldDeleteInstance)
	} catch (error) {
		handleError(toError(error))
	} finally {
		submitLoading.value = false
	}
}
async function onImageUpload(file: File) {
	const imageExtensionByType: Partial<Record<string, Labrinth.Images.v3.ImageExtension>> = {
		'image/gif': 'gif',
		'image/jpeg': 'jpeg',
		'image/png': 'png',
		'image/webp': 'webp',
	}
	const extension = imageExtensionByType[file.type]
	if (!extension) {
		throw new Error(formatMessage(messages.invalidImageType))
	}
	if (file.size > 1024 * 1024) {
		throw new Error(formatMessage(messages.imageTooLarge))
	}

	const image = await client.labrinth.images_v3.uploadImage(file, extension, {
		context: 'report',
	}).promise
	uploadedImageIDs.value.push(image.id)
	return image.url
}
function handleCancel() {
	if (reportMode.value && !reportOnly.value) {
		reportMode.value = false
		void nextTick(() => forceCheckTableScroll())
		return
	}
	hide()
}
function handleHide() {
	resetReportState()
	show_ads_window()
}
function resetReportState() {
	reportMode.value = false
	reportOnly.value = false
	reportReason.value = 'malicious'
	additionalContext.value = ''
	deleteInstance.value = true
	submitLoading.value = false
	uploadedImageIDs.value = []
}
function show(
	previewValue: SharedInstanceInstallPreview,
	installValue: () => void | Promise<void>,
	event?: MouseEvent,
) {
	resetReportState()
	install.value = installValue
	showPreview(previewValue, event)
}
function showReport(previewValue: SharedInstanceInstallPreview, event?: MouseEvent) {
	resetReportState()
	reportMode.value = true
	reportOnly.value = true
	install.value = () => {}
	showPreview(previewValue, event)
}
function showPreview(previewValue: SharedInstanceInstallPreview, event?: MouseEvent) {
	preview.value = previewValue
	hide_ads_window()
	modal.value?.show(event)
	void nextTick(() => forceCheckTableScroll())
}
function hide() {
	modal.value?.hide()
}

const messages = defineMessages({
	installToPlay: { id: 'app.modal.install-to-play.header', defaultMessage: 'Install to play' },
	reportSharedInstance: {
		id: 'app.modal.install-to-play.report-shared-instance-header',
		defaultMessage: 'Report shared instance',
	},
	reportSubmitted: {
		id: 'app.modal.install-to-play.report-submitted',
		defaultMessage: 'Report submitted',
	},
	sharedInstanceContent: {
		id: 'app.modal.install-to-play.shared-instance-content',
		defaultMessage: 'Shared instance content',
	},
	inviteWarning: {
		id: 'app.modal.install-to-play.invite-warning',
		defaultMessage:
			'This invite was created by another Modrinth user, not Modrinth. Only accept invites from people you trust.',
	},
	reportDescription: {
		id: 'app.modal.install-to-play.report-description',
		defaultMessage:
			'Use this form to report instances that may violate our <rules-link>Rules</rules-link> or <terms-link>Terms of Use</terms-link>.',
	},
	supportAndBugReports: {
		id: 'app.modal.install-to-play.report-support-and-bugs',
		defaultMessage:
			'For support requests, contact our <support-link>support team</support-link>. For bug reports, open a <github-link>GitHub issue</github-link>.',
	},
	legalClaims: {
		id: 'app.modal.install-to-play.report-legal-claims',
		defaultMessage:
			'For DMCA notices or other legal claims, see our <copyright-link>Copyright Policy</copyright-link>.',
	},
	contentYouAreReporting: {
		id: 'app.modal.install-to-play.content-you-are-reporting',
		defaultMessage: 'Instance you’re reporting',
	},
	reportReason: {
		id: 'app.modal.install-to-play.report-reason',
		defaultMessage: 'Which rule does this instance violate?',
	},
	maliciousReason: {
		id: 'app.modal.install-to-play.report-reason.malicious',
		defaultMessage: 'Malicious',
	},
	inappropriateReason: {
		id: 'app.modal.install-to-play.report-reason.inappropriate',
		defaultMessage: 'Inappropriate',
	},
	spamReason: {
		id: 'app.modal.install-to-play.report-reason.spam',
		defaultMessage: 'Spam',
	},
	additionalContext: {
		id: 'app.modal.install-to-play.additional-context',
		defaultMessage: 'Additional context',
	},
	additionalContextPlaceholder: {
		id: 'app.modal.install-to-play.additional-context-placeholder',
		defaultMessage: 'Include links and images if possible and relevant',
	},
	invalidImageType: {
		id: 'app.modal.install-to-play.report-image-invalid-type',
		defaultMessage: 'File is not an accepted image type',
	},
	imageTooLarge: {
		id: 'app.modal.install-to-play.report-image-too-large',
		defaultMessage: 'File exceeds the 1 MiB size limit',
	},
	deleteInstance: {
		id: 'app.modal.install-to-play.delete-instance',
		defaultMessage: 'Delete instance',
	},
	unknownFilesWarning: {
		id: 'app.modal.install-to-play.unknown-files-warning',
		defaultMessage: 'Unknown files warning',
	},
	unknownFilesDescription: {
		id: 'app.modal.install-to-play.shared-instance-unknown-files-description',
		defaultMessage:
			'This shared instance contains files that aren’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	unrecognizedFiles: {
		id: 'app.modal.install-to-play.unrecognized-files',
		defaultMessage: 'Unrecognized files',
	},
	reviewedFiles: {
		id: 'app.modal.install-to-play.reviewed-files',
		defaultMessage:
			'A file is only reviewed if it’s published to Modrinth, regardless of its file format (including .mrpack).',
	},
	installAnyway: {
		id: 'app.modal.install-to-play.install-anyway',
		defaultMessage: 'Install anyway',
	},
	dontInstall: {
		id: 'app.modal.install-to-play.external-files-dont-install',
		defaultMessage: "Don't install",
	},
	installButton: { id: 'app.modal.install-to-play.install-button', defaultMessage: 'Install' },
})

const externalFileColumns = computed<TableColumn<ExternalFileColumn>[]>(() => [
	{
		key: 'name',
		label: formatMessage(messages.unrecognizedFiles),
		cellClass: '!h-12',
	},
])

defineExpose({ show, showReport, hide })
</script>
