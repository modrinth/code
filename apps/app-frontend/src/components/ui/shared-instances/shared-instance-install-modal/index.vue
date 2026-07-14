<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.installToPlay)"
		:closable="true"
		:on-hide="handleHide"
		no-padding
	>
		<div v-if="preview" class="flex flex-col gap-6 max-w-[500px] p-6 pb-2">
			<Admonition
				:type="reportMode ? 'info' : 'warning'"
				:header="reportMode ? undefined : formatMessage(messages.trustWarningHeader)"
			>
				<template v-if="reportMode">
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
				</template>
				<template v-else>
					{{ formatMessage(messages.trustWarningDescription) }}
				</template>
			</Admonition>
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
						<Combobox
							v-model="reportReason"
							:options="reportReasonOptions"
						/>
					</div>
					<div class="flex flex-col gap-2.5">
						<label for="shared-instance-report-context" class="font-semibold text-contrast">
							{{ formatMessage(messages.additionalContext) }}
						</label>
						<StyledInput
							id="shared-instance-report-context"
							v-model="additionalContext"
							multiline
							:rows="5"
							:placeholder="formatMessage(messages.additionalContextPlaceholder)"
							wrapper-class="w-full"
						/>
					</div>
					<Checkbox v-model="blockUser" :label="formatMessage(messages.blockUser)" />
				</div>
			</Transition>
			<div
				v-if="!reportMode && preview.externalFileCount"
				class="flex items-center gap-2 text-orange"
			>
				<IssuesIcon class="h-5 w-5 flex-none" />
				<span>{{
					formatMessage(messages.externalFileWarning, { count: preview.externalFileCount })
				}}</span>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-between gap-2">
				<div>
					<ButtonStyled v-if="!reportMode" color="red" type="transparent">
						<button @click="reportMode = true">
							<ReportIcon />{{ formatMessage(commonMessages.reportButton) }}
						</button>
					</ButtonStyled>
				</div>
				<div class="flex gap-2">
					<ButtonStyled type="outlined">
						<button class="!border" @click="handleCancel">
							<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="reportMode" color="brand">
						<button>
							<SendIcon />{{ formatMessage(commonMessages.reportButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled v-else color="brand">
						<button @click="accept">
							<DownloadIcon />{{ formatMessage(messages.installButton) }}
						</button>
					</ButtonStyled>
				</div>
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
import { DownloadIcon, IssuesIcon, ReportIcon, SendIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	AutoLink,
	ButtonStyled,
	Checkbox,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	IntlFormatted,
	ModpackContentModal,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads'
import type { SharedInstanceInstallPreview } from '@/helpers/install'

import SharedInstanceInstallSummary from './shared-instance-install-summary.vue'
import { useSharedInstancePreviewContent } from './use-shared-instance-preview-content'

const modal = ref<InstanceType<typeof NewModal>>()
const contentModal = ref<InstanceType<typeof ModpackContentModal>>()
const preview = ref<SharedInstanceInstallPreview | null>(null)
const install = ref<() => void | Promise<void>>(() => {})
const reportMode = ref(false)
type ReportReason = 'malicious' | 'illegal' | 'other'
const reportReason = ref<ReportReason>('malicious')
const additionalContext = ref('')
const blockUser = ref(false)
const { formatMessage } = useVIntl()
const { load } = useSharedInstancePreviewContent()
const reportReasonOptions = computed<ComboboxOption<ReportReason>[]>(() => [
	{ value: 'malicious', label: formatMessage(messages.maliciousReason) },
	{ value: 'illegal', label: formatMessage(messages.illegalReason) },
	{ value: 'other', label: formatMessage(messages.otherReason) },
])

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
function handleCancel() {
	if (reportMode.value) {
		reportMode.value = false
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
	reportReason.value = 'malicious'
	additionalContext.value = ''
	blockUser.value = false
}
function show(
	previewValue: SharedInstanceInstallPreview,
	installValue: () => void | Promise<void>,
	event?: MouseEvent,
) {
	resetReportState()
	preview.value = previewValue
	install.value = installValue
	hide_ads_window()
	modal.value?.show(event)
}
function hide() {
	modal.value?.hide()
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
	reportDescription: {
		id: 'app.modal.install-to-play.report-description',
		defaultMessage:
			'Use this form to report content that may violate our <rules-link>Rules</rules-link> or <terms-link>Terms of Use</terms-link>.',
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
		defaultMessage: 'Content you’re reporting',
	},
	reportReason: {
		id: 'app.modal.install-to-play.report-reason',
		defaultMessage: 'Which rule does this content violate?',
	},
	maliciousReason: {
		id: 'app.modal.install-to-play.report-reason.malicious',
		defaultMessage: 'Malicious',
	},
	illegalReason: {
		id: 'app.modal.install-to-play.report-reason.illegal',
		defaultMessage: 'Illegal',
	},
	otherReason: {
		id: 'app.modal.install-to-play.report-reason.other',
		defaultMessage: 'Other',
	},
	additionalContext: {
		id: 'app.modal.install-to-play.additional-context',
		defaultMessage: 'Additional context',
	},
	additionalContextPlaceholder: {
		id: 'app.modal.install-to-play.additional-context-placeholder',
		defaultMessage: 'Include links and images if possible and relevant',
	},
	blockUser: {
		id: 'app.modal.install-to-play.block-user',
		defaultMessage: 'Block this user',
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
