<template>
	<div class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(commonMessages.detailsLabel) }}</h2>
		<div class="flex flex-col gap-3 font-semibold [&>div]:flex [&>div]:gap-2 [&>div]:items-center">
			<div>
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
								<ExternalIcon aria-hidden="true" class="external-icon ml-1 mt-[-1px] inline" />
							</a>
							<span
								v-else-if="
									project.license.id === 'LicenseRef-All-Rights-Reserved' ||
									!project.license.id.includes('LicenseRef')
								"
							>
								{{ licenseIdDisplay }}
							</span>
							<span v-else>{{ licenseIdDisplay }}</span>
						</template>
					</IntlFormatted>
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
			<div v-if="hasVersions && project.updated" v-tooltip="formatDateTime(project.updated)">
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
import { BookTextIcon, CalendarIcon, ExternalIcon, ScaleIcon, VersionIcon } from '@modrinth/assets'
import { capitalizeString } from '@modrinth/utils'
import { computed } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import { IntlFormatted } from '../base'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const props = defineProps<{
	project: {
		id: string
		published: string
		updated: string
		approved: string
		queued: string
		status: string
		license: {
			id: string
			url: string
		}
	}
	linkTarget: string
	hasVersions: boolean
}>()

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

const licenseIdDisplay = computed(() => {
	const id = props.project.license.id

	if (id === 'LicenseRef-All-Rights-Reserved') {
		return 'ARR'
	} else if (id.includes('LicenseRef')) {
		return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
	} else {
		return id
	}
})

const messages = defineMessages({
	licensed: {
		id: 'project.about.details.licensed',
		defaultMessage: 'Licensed {license}',
	},
})
</script>
