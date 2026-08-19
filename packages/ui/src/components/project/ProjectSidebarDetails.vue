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
			class="flex flex-col gap-3 [&>div>svg]:shrink-0 [&>div>svg]:mt-[1px] [&>div]:flex [&>div]:gap-2 [&>div]:items-start [&>div>div]:min-w-0"
		>
			<div v-if="photosensitivityDisclosure" class="text-orange">
				<EyeIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.photosensitivityTitle)) }}
					</span>
					<span v-if="photosensitivityDisclosure.note" class="text-sm text-secondary">
						<BasicMarkdownText :text="photosensitivityDisclosure.note" :target="linkTarget" />
					</span>
				</div>
			</div>
			<div v-if="aiDisclosure">
				<SparklesIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(aiGeneratedLabel) }}
					</span>
					<span v-if="aiDisclosure.note" class="text-sm text-secondary">
						<BasicMarkdownText :text="aiDisclosure.note" :target="linkTarget" />
					</span>
				</div>
			</div>
			<div v-if="advertisingDisclosure">
				<MegaphoneIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.advertisingTitle)) }}
					</span>
					<span v-if="advertisingDisclosure.note" class="text-sm text-secondary">
						<BasicMarkdownText :text="advertisingDisclosure.note" :target="linkTarget" />
					</span>
				</div>
			</div>
			<div v-if="paidFeaturesDisclosure">
				<CircleDollarSignIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.paidFeatures)) }}
					</span>
					<span
						v-for="(feature, index) in paidFeaturesDisclosure.features"
						:key="`${feature}-${index}`"
						class="text-sm text-secondary"
					>
						<BasicMarkdownText :text="feature" :target="linkTarget" />
					</span>
				</div>
			</div>
			<div v-if="telemetryDisclosure">
				<RadioTowerIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{
							capitalizeString(
								formatMessage(messages.telemetryTitle, {
									consent: telemetryDisclosure.consent,
								}),
							)
						}}
					</span>
					<span
						v-for="(entry, index) in telemetryDisclosure.data_collected"
						:key="`${entry}-${index}`"
						class="text-sm text-secondary"
					>
						<BasicMarkdownText :text="entry" :target="linkTarget" />
					</span>
				</div>
			</div>
			<div v-if="systemInteractionsDisclosure">
				<CircuitBoardIcon aria-hidden="true" />
				<div class="flex flex-col gap-1">
					<span>
						{{ capitalizeString(formatMessage(messages.systemInteractionsTitle)) }}
					</span>
					<span v-if="systemInteractionsDisclosure.note" class="text-sm text-secondary">
						<BasicMarkdownText :text="systemInteractionsDisclosure.note" :target="linkTarget" />
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
			<div v-if="derivativeWorkDisclosure">
				<GitForkIcon aria-hidden="true" />
				<div class="flex flex-col gap-2">
					<span>
						{{ capitalizeString(formatMessage(messages.derivativeWork)) }}
					</span>
					<div
						v-for="(source, index) in visibleDerivativeSources"
						:key="`${source.label}-${index}`"
						class="flex flex-col gap-1"
					>
						<a
							v-if="source.link"
							:href="source.link"
							:target="linkTarget"
							rel="noopener nofollow ugc"
							class="text-blue text-sm min-w-0 break-words hover:underline"
						>
							{{ source.label }}
							<ExternalIcon />
						</a>
						<span v-else class="text-sm">
							<BasicMarkdownText :text="source.label" :target="linkTarget" />
						</span>
						<span v-if="source.note" class="text-sm text-secondary">
							<BasicMarkdownText :text="source.note" :target="linkTarget" />
						</span>
					</div>
					<button
						v-if="hasMoreDerivativeSources"
						type="button"
						class="flex w-fit items-center gap-1 border-none bg-transparent p-0 text-sm font-semibold text-secondary cursor-pointer active:scale-95 transition-transform"
						@click="showAllDerivativeSources = !showAllDerivativeSources"
					>
						<DropdownIcon
							class="h-4 w-4 transition-transform"
							:class="{ 'rotate-180': showAllDerivativeSources }"
						/>
						{{
							showAllDerivativeSources
								? formatMessage(messages.showLessDerivativeSources)
								: formatMessage(messages.showMoreDerivativeSources, {
										count: hiddenDerivativeSourcesCount,
									})
						}}
					</button>
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
	CircuitBoardIcon,
	DropdownIcon,
	ExternalIcon,
	EyeIcon,
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
import { defineMessage, defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import { commonMessages } from '../../utils/common-messages'
import { getActiveDisclosures } from '../../utils/disclosures'
import { Avatar, BasicMarkdownText, IntlFormatted } from '../base'
import { NewModal } from '../modal'

const LICENSE_STALE_TIME = 1000 * 60 * 10
const DISCLOSURE_STALE_TIME = 1000 * 60 * 5

const { formatMessage, locale } = useVIntl()
const { labrinth } = injectModrinthClient()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const AI_USE_ORDER: Labrinth.Projects.v3.AiUsage[] = ['code', 'assets', 'text', 'functionality']

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
		defaultMessage: 'Contains AI-generated {types}',
	},
	derivativeWork: {
		id: 'project.disclosure.derivative-work.title',
		defaultMessage: 'This is a derivative work of:',
	},
	showMoreDerivativeSources: {
		id: 'project.disclosure.derivative-work.show-more',
		defaultMessage: 'Show {count} more',
	},
	showLessDerivativeSources: {
		id: 'project.disclosure.derivative-work.show-fewer',
		defaultMessage: 'Show fewer',
	},
	telemetryTitle: {
		id: 'project.disclosure.telemetry.title',
		defaultMessage:
			'Contains {consent, select, opt_in {opt-in telemetry} opt_out {opt-out telemetry} always_active {always-active telemetry} other {telemetry}}',
	},
	photosensitivityTitle: {
		id: 'project.disclosure.photosensitivity.title',
		defaultMessage: 'Photosensitivity warning',
	},
	systemInteractionsTitle: {
		id: 'project.disclosure.system-interactions.title',
		defaultMessage: 'Contains external system interactions',
	},
})

const { data: disclosuresResponse } = useQuery({
	queryKey: computed(() => ['project', 'disclosures', 'v3', props.project.id] as const),
	queryFn: () => labrinth.projects_v3.getDisclosures(props.project.id),
	staleTime: DISCLOSURE_STALE_TIME,
})

const disclosures = computed(() => getActiveDisclosures(disclosuresResponse.value?.disclosures))

function findDisclosure<T extends Labrinth.Projects.v3.ProjectDisclosureType>(type: T) {
	return disclosures.value.find(
		(d): d is Labrinth.Projects.v3.ProjectDisclosureOf<T> => d.type === type,
	)
}

const aiDisclosure = computed(() => findDisclosure('ai_content'))
const advertisingDisclosure = computed(() => findDisclosure('advertisements'))
const paidFeaturesDisclosure = computed(() => findDisclosure('paid_features'))
const telemetryDisclosure = computed(() => findDisclosure('telemetry'))
const derivativeWorkDisclosure = computed(() => findDisclosure('derivative_work'))
const photosensitivityDisclosure = computed(() => findDisclosure('epilepsy_triggers'))
const systemInteractionsDisclosure = computed(() => findDisclosure('system_interactions'))

const DERIVATIVE_SOURCES_PREVIEW_LIMIT = 3
const showAllDerivativeSources = ref(false)

const visibleDerivativeSources = computed(() => {
	const sources = derivativeWorkDisclosure.value?.sources ?? []
	if (showAllDerivativeSources.value || sources.length <= DERIVATIVE_SOURCES_PREVIEW_LIMIT) {
		return sources
	}
	return sources.slice(0, DERIVATIVE_SOURCES_PREVIEW_LIMIT)
})

const hasMoreDerivativeSources = computed(
	() => (derivativeWorkDisclosure.value?.sources.length ?? 0) > DERIVATIVE_SOURCES_PREVIEW_LIMIT,
)

const hiddenDerivativeSourcesCount = computed(() =>
	Math.max(
		0,
		(derivativeWorkDisclosure.value?.sources.length ?? 0) - DERIVATIVE_SOURCES_PREVIEW_LIMIT,
	),
)

const aiUseLabels = {
	code: defineMessage({
		id: 'project.disclosure.ai-generated-content.use.code',
		defaultMessage: 'code',
	}),
	assets: defineMessage({
		id: 'project.disclosure.ai-generated-content.use.assets',
		defaultMessage: 'assets',
	}),
	text: defineMessage({
		id: 'project.disclosure.ai-generated-content.use.text',
		defaultMessage: 'text',
	}),
	functionality: defineMessage({
		id: 'project.disclosure.ai-generated-content.use.functionality',
		defaultMessage: 'functionality',
	}),
	content: defineMessage({
		id: 'project.disclosure.ai-generated-content.use.content',
		defaultMessage: 'content',
	}),
} as const

const aiGeneratedLabel = computed(() => {
	const disclosure = aiDisclosure.value
	if (!disclosure) {
		return ''
	}

	const orderedUses = AI_USE_ORDER.filter((use) => disclosure.uses.includes(use))
	const types =
		orderedUses.length === 0
			? formatMessage(aiUseLabels.content)
			: new Intl.ListFormat(locale.value, {
					style: 'long',
					type: 'conjunction',
				}).format(orderedUses.map((use) => formatMessage(aiUseLabels[use])))

	return formatMessage(messages.aiGeneratedContent, { types })
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
