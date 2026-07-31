<template>
	<div class="flex flex-col gap-3">
		<NewModal
			ref="modalLicense"
			:header="project.license.name ? project.license.name : formatMessage(messages.licenseTitle)"
		>
			<template #title>
				<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" no-shadow />
				<span class="text-lg font-extrabold text-contrast">
					{{ project.license.name ? project.license.name : formatMessage(messages.licenseTitle) }}
				</span>
			</template>
			<div class="markdown-body" v-html="licenseHtml" />
		</NewModal>
		<h2 class="text-lg m-0">{{ formatMessage(commonMessages.detailsLabel) }}</h2>
		<div
			class="flex flex-col gap-3 [&>div>svg]:shrink-0 [&>div>svg]:mt-[1px] [&>div]:flex [&>div]:gap-2 [&>div]:items-start"
		>
			<div>
				<SparklesIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.aiGeneratedContent, { type: 'code' })) }}
					</span>
					<span class="text-sm text-secondary">
						The Chinese and Arabic translations are AI-generated
					</span>
				</div>
			</div>
			<div>
				<MegaphoneIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.advertisingTitle)) }}
					</span>
					<span class="text-sm text-secondary"> Title screen has Essential promotion </span>
				</div>
			</div>
			<div>
				<CircleDollarSignIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.paidFeatures)) }}
					</span>
					<span class="text-sm text-secondary"> Cosmetics available as Patreon reward </span>
				</div>
			</div>
			<div>
				<RadioTowerIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.telemetryTitle, { consent: 'opt_out' })) }}
					</span>
					<span class="text-sm text-secondary">
						Update checker provides anonymous launch analytics to Modrinth
					</span>
					<span class="text-sm text-blue flex items-center gap-1">
						View privacy policy <ExternalIcon />
					</span>
				</div>
			</div>
			<div v-if="!hideLicense">
				<BookTextIcon aria-hidden="true" />
				<div>
					<IntlFormatted :message-id="messages.licensed">
						<template #~license>
							<a
								v-if="project.license.url"
								class="text-link hover:underline"
								:href="project.license.url"
								:target="linkTarget"
								rel="noopener nofollow ugc"
							>
								{{ licenseIdDisplay }}
							</a>
							<span
								v-else-if="canOpenLicenseModal"
								class="text-link hover:underline cursor-pointer"
								@mouseenter="enableLicenseFetch"
								@click="(event) => openLicenseModal(event)"
							>
								{{ licenseIdDisplay }}
							</span>
							<span v-else>{{ licenseIdDisplay }}</span>
						</template>
					</IntlFormatted>
				</div>
			</div>
			<div>
				<GitForkIcon aria-hidden="true" />
				<div class="flex flex-col gap-2">
					<span>
						{{ capitalizeString(formatMessage(messages.derivativeWork)) }}
					</span>
					<div class="flex flex-col gap-1">
						<span class="text-blue text-sm flex items-center gap-1">
							Modification Menu <ExternalIcon />
						</span>
						<span class="text-sm text-secondary"> Forked to add Fun Mode </span>
					</div>
				</div>
			</div>
			<div v-if="showFollowers">
				<HeartIcon aria-hidden="true" />
				<div>
					{{ formatMessage(commonMessages.projectFollowers, { count: project.followers }) }}
				</div>
			</div>
			<div v-if="project.approved" v-tooltip="formatDateTime(project.approved)">
				<CalendarIcon aria-hidden="true" />
				<div>
					{{
						capitalizeString(
							formatMessage(commonMessages.projectPublished, { date: publishedDate }),
						)
					}}
				</div>
			</div>
			<div v-else v-tooltip="formatDateTime(project.published)">
				<CalendarIcon aria-hidden="true" />
				<div>
					{{
						capitalizeString(formatMessage(commonMessages.projectCreated, { date: createdDate }))
					}}
				</div>
			</div>
			<div
				v-if="project.status === 'processing' && project.queued"
				v-tooltip="formatDateTime(project.queued)"
			>
				<ScaleIcon aria-hidden="true" />
				<div>
					{{
						capitalizeString(
							formatMessage(commonMessages.projectSubmitted, { date: submittedDate }),
						)
					}}
				</div>
			</div>
			<div
				v-if="project.versions.length > 0 && project.updated"
				v-tooltip="formatDateTime(project.updated)"
			>
				<VersionIcon aria-hidden="true" />
				<div>
					{{
						capitalizeString(formatMessage(commonMessages.projectUpdated, { date: updatedDate }))
					}}
				</div>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BookTextIcon,
	CalendarIcon,
	CircleDollarSignIcon,
	ExternalIcon,
	GitForkIcon,
	HeartIcon,
	MegaphoneIcon,
	RadioTowerIcon,
	ScaleIcon,
	SparklesIcon,
	VersionIcon,
} from '@modrinth/assets'
import { capitalizeString, renderString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import { commonMessages } from '../../utils/common-messages'
import { Avatar, IntlFormatted } from '../base'
import { NewModal } from '../modal'

const LICENSE_STALE_TIME = 1000 * 60 * 10

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const props = defineProps<{
	project: Labrinth.Projects.v2.Project
	linkTarget: string
	hideLicense?: boolean
	showFollowers?: boolean
}>()

const modalLicense = useTemplateRef('modalLicense')
const licenseFetchEnabled = ref(false)

const messages = defineMessages({
	licensed: {
		id: 'project.about.details.licensed',
		defaultMessage: 'Licensed {license}',
	},
	licenseErrorMessage: {
		id: 'project.license.error',
		defaultMessage: 'License text could not be retrieved.',
	},
	licenseTitle: {
		id: 'project.license.title',
		defaultMessage: 'License',
	},
	loadingLicenseText: {
		id: 'project.license.loading',
		defaultMessage: 'Loading license text...',
	},
	advertisingTitle: {
		id: 'project.disclosure.advertising.title',
		defaultMessage: 'Contains advertising',
	},
	paidFeatures: {
		id: 'project.disclosure.paid-features.title',
		defaultMessage: 'Contains paid features',
	},
	aiGeneratedContent: {
		id: 'project.disclosure.ai-generated-content.title',
		defaultMessage:
			'Contains AI-generated {type, select, code {code} assets {assets} code_assets {code and assets} text {text} other {content}}',
	},
	derivativeWork: {
		id: 'project.disclosure.derivative-work.title',
		defaultMessage: 'This is a derivative work of:',
	},
	telemetryTitle: {
		id: 'project.disclosure.telemetry.title',
		defaultMessage:
			'Contains {consent, select, opt_in {opt-in telemetry} opt_out {opt-out telemetry} other {telemetry}}',
	},
})

const createdDate = computed(() =>
	props.project.published ? formatRelativeTime(props.project.published) : 'unknown',
)
const submittedDate = computed(() =>
	props.project.queued ? formatRelativeTime(props.project.queued) : 'unknown',
)
const publishedDate = computed(() =>
	props.project.approved ? formatRelativeTime(props.project.approved) : 'unknown',
)
const updatedDate = computed(() =>
	props.project.updated ? formatRelativeTime(props.project.updated) : 'unknown',
)

const licenseId = computed(() => props.project.license.id)

const licenseIdDisplay = computed(() => {
	const id = licenseId.value

	if (id === 'LicenseRef-All-Rights-Reserved') {
		return 'ARR'
	} else if (id.includes('LicenseRef')) {
		return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
	} else {
		return id
	}
})

const canOpenLicenseModal = computed(() => {
	if (props.hideLicense || props.project.license.url) {
		return false
	}

	const id = licenseId.value
	return id === 'LicenseRef-All-Rights-Reserved' || !id.includes('LicenseRef')
})

const { data: licenseBody, isError: isLicenseError } = useQuery({
	queryKey: computed(() => ['license', 'v2', licenseId.value] as const),
	queryFn: async () => {
		const text = await labrinth.tags_v2.getLicenseText(licenseId.value)
		return text.body
	},
	enabled: computed(() => canOpenLicenseModal.value && licenseFetchEnabled.value),
	staleTime: LICENSE_STALE_TIME,
})

const licenseHtml = computed(() => {
	if (licenseBody.value) {
		return renderString(licenseBody.value)
	}

	if (isLicenseError.value || licenseBody.value === '') {
		return renderString(formatMessage(messages.licenseErrorMessage))
	}

	return formatMessage(messages.loadingLicenseText)
})

function enableLicenseFetch() {
	if (!canOpenLicenseModal.value) {
		return
	}

	licenseFetchEnabled.value = true
}

function openLicenseModal(event?: MouseEvent) {
	enableLicenseFetch()
	modalLicense.value?.show(event)
}
</script>
