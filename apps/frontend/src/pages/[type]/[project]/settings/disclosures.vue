<script setup lang="ts">
import {
	ConfirmLeaveModal,
	defineMessages,
	injectModrinthClient,
	injectProjectPageContext,
	IntlFormatted,
	type MessageDescriptor,
	normalizeChildren,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { TeamMemberPermission } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, watch } from 'vue'

import {
	AdvertisingDisclosureCard,
	AiDisclosureCard,
	ArchivedDisclosureCard,
	DerivativeDisclosureCard,
	type DisclosureFormIssue,
	disclosuresToForm,
	getDisclosureFormIssues,
	PaidFeaturesDisclosureCard,
	PhotosensitivityDisclosureCard,
	SystemInteractionsDisclosureCard,
	TelemetryDisclosureCard,
	toCachedDisclosures,
	toModifyRequest,
} from '~/components/ui/project-settings/disclosures'

const DISCLOSURE_QUERY_STALE_TIME = 1000 * 60 * 5

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()
const { projectV2: project, currentMember } = injectProjectPageContext()
const queryClient = useQueryClient()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.content-disclosures',
		defaultMessage: 'Content disclosures',
	},
	description: {
		id: 'project.settings.disclosures.description',
		defaultMessage: `You must add any applicable content disclosures to your project in compliance with Modrinth's <rules>Content Rules</rules>.`,
	},
	noPermission: {
		id: 'project.settings.disclosures.save-blocked.no-permission',
		defaultMessage: `You don't have permission to edit this project's disclosures.`,
	},
})

const issueMessages = defineMessages({
	'advertising-note': {
		id: 'project.settings.disclosures.save-blocked.advertising-note',
		defaultMessage: 'Advertising disclosure requires explanation.',
	},
	'paid-features-empty': {
		id: 'project.settings.disclosures.save-blocked.paid-features-empty',
		defaultMessage: 'Paid features disclosure must list at least one paid feature.',
	},
	'telemetry-empty': {
		id: 'project.settings.disclosures.save-blocked.telemetry-empty',
		defaultMessage: 'Telemetry disclosure must list at least one type of data collected.',
	},
	'derivative-empty': {
		id: 'project.settings.disclosures.save-blocked.derivative-empty',
		defaultMessage: 'Derivative works disclosure must list at least one source work.',
	},
	'derivative-source-label': {
		id: 'project.settings.disclosures.save-blocked.derivative-source-label',
		defaultMessage: 'Derivative work sources must have a name.',
	},
	'photosensitivity-note': {
		id: 'project.settings.disclosures.save-blocked.photosensitivity-note',
		defaultMessage: 'Photosensitivity warning disclosure must include a description.',
	},
	'system-interactions-note': {
		id: 'project.settings.disclosures.save-blocked.system-interactions-note',
		defaultMessage: 'External system interactions disclosure must include a description.',
	},
}) satisfies Record<DisclosureFormIssue, MessageDescriptor>

useProjectSettingsHeadTitle(messages.title)

const disclosuresQueryKey = computed(
	() => ['project', 'disclosures', 'v3', project.value.id] as const,
)

const { data: disclosuresResponse } = useQuery({
	queryKey: disclosuresQueryKey,
	queryFn: () => labrinth.projects_v3.getDisclosures(project.value.id),
	staleTime: DISCLOSURE_QUERY_STALE_TIME,
})

const hasPermission = computed(
	() => !!((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
)

const { saved, current, saving, hasChanges, reset, save } = useSavable(
	() => disclosuresToForm(disclosuresResponse.value?.disclosures ?? []),
	async () => {
		if (!hasPermission.value || !canSave.value) {
			throw new Error('Disclosures form is not valid')
		}

		const previous = disclosuresToForm(disclosuresResponse.value?.disclosures ?? [])
		const request = toModifyRequest(current.value, previous)
		await labrinth.projects_v3.modifyDisclosures(project.value.id, request)
		queryClient.setQueryData(disclosuresQueryKey.value, {
			disclosures: toCachedDisclosures(request.set),
		})
	},
)

watch(
	() => disclosuresResponse.value,
	(value, previous) => {
		if (value && (!previous || !hasChanges.value)) {
			reset()
		}
	},
)

const issues = computed(() => getDisclosureFormIssues(current.value))

const canSave = computed(() => hasPermission.value && issues.value.length === 0)

const saveDisabledReason = computed(() => {
	if (!hasPermission.value) {
		// should never come up but y'never know
		return formatMessage(messages.noPermission)
	}
	return issues.value.map((issue) => formatMessage(issueMessages[issue]))
})

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)
</script>

<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<h2 class="m-0 text-2xl font-semibold">
			{{ formatMessage(messages.title) }}
		</h2>
		<p class="mb-4 mt-2">
			<IntlFormatted :message-id="messages.description">
				<template #rules="{ children }">
					<nuxt-link to="/legal/rules" target="_blank" class="underline hover:text-contrast">
						<component :is="() => normalizeChildren(children)" />
					</nuxt-link>
				</template>
			</IntlFormatted>
		</p>
		<div class="flex flex-col gap-4">
			<AiDisclosureCard v-model="current.ai" :disabled="!hasPermission" />
			<AdvertisingDisclosureCard v-model="current.advertising" :disabled="!hasPermission" />
			<PaidFeaturesDisclosureCard v-model="current.paidFeatures" :disabled="!hasPermission" />
			<TelemetryDisclosureCard v-model="current.telemetry" :disabled="!hasPermission" />
			<DerivativeDisclosureCard v-model="current.derivative" :disabled="!hasPermission" />
			<PhotosensitivityDisclosureCard
				v-model="current.photosensitivity"
				:disabled="!hasPermission"
			/>
			<SystemInteractionsDisclosureCard
				v-model="current.systemInteractions"
				:disabled="!hasPermission"
			/>
			<ArchivedDisclosureCard
				v-model="current.archived"
				:disabled="!hasPermission"
				:project-title="project.title"
			/>
		</div>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-save="canSave"
			:save-disabled-reason="saveDisabledReason"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>
