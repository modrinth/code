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
		<div class="flex flex-col gap-3 [&>div]:flex [&>div]:gap-2 [&>div]:items-center">
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
import { BookTextIcon, CalendarIcon, HeartIcon, ScaleIcon, VersionIcon } from '@modrinth/assets'
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
