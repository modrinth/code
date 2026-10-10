<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'

import ReviewPanel from '~/components/project-review/review-panel/index.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import AdvertisingDisclosureCard from './AdvertisingDisclosureCard.vue'
import AiDisclosureCard from './AiDisclosureCard.vue'
import AiFunctionalityDisclosureCard from './AiFunctionalityDisclosureCard.vue'
import ArchivedDisclosureCard from './ArchivedDisclosureCard.vue'
import DerivativeDisclosureCard from './DerivativeDisclosureCard.vue'
import PaidFeaturesDisclosureCard from './PaidFeaturesDisclosureCard.vue'
import PhotosensitivityDisclosureCard from './PhotosensitivityDisclosureCard.vue'
import SystemInteractionsDisclosureCard from './SystemInteractionsDisclosureCard.vue'
import TelemetryDisclosureCard from './TelemetryDisclosureCard.vue'
import type { DisclosureLockStatus } from './types'
import type { useDisclosureEditor } from './use-disclosure-editor'

const props = withDefaults(
	defineProps<{
		editor: ReturnType<typeof useDisclosureEditor>
		projectTitle: string
		hideDescription?: boolean
		variant?: 'settings' | 'review'
	}>(),
	{ hideDescription: false, variant: 'settings' },
)
const { current, disclosureUpdateProps, setDisclosureLockStatus, isDisclosureVisible } =
	props.editor
const panels = injectReviewPanels(null)

function reviewTitle(key: string): string | undefined {
	return props.variant === 'review'
		? panels?.resolve({ kind: 'disclosure', key })?.panel.title
		: undefined
}

const disclosureIssueTargets: Labrinth.Threads.v3.ThreadIssueTarget['type'][] = [
	'add_project_disclosures',
	'remove_project_disclosures',
	'modify_project_disclosure',
	'modify_project_disclosure_note',
]
</script>

<template>
	<div class="flex flex-col gap-4">
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('ai_content')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'ai' },
						}
					: {}
			"
		>
			<AiDisclosureCard
				v-model="current.ai"
				:hide-description="hideDescription"
				:title="reviewTitle('ai')"
				:variant="variant"
				v-bind="disclosureUpdateProps('ai_content')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('ai_content', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="ai_content"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('ai_functionality')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'ai-functionality' },
						}
					: {}
			"
		>
			<AiFunctionalityDisclosureCard
				v-model="current.aiFunctionality"
				:hide-description="hideDescription"
				:title="reviewTitle('ai-functionality')"
				:variant="variant"
				v-bind="disclosureUpdateProps('ai_functionality')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('ai_functionality', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="ai_functionality"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('advertisements')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'ads' },
						}
					: {}
			"
		>
			<AdvertisingDisclosureCard
				v-model="current.advertising"
				:hide-description="hideDescription"
				:title="reviewTitle('ads')"
				:variant="variant"
				v-bind="disclosureUpdateProps('advertisements')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('advertisements', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="advertisements"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('paid_features')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'paid-features' },
						}
					: {}
			"
		>
			<PaidFeaturesDisclosureCard
				v-model="current.paidFeatures"
				:hide-description="hideDescription"
				:title="reviewTitle('paid-features')"
				:variant="variant"
				v-bind="disclosureUpdateProps('paid_features')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('paid_features', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="paid_features"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('telemetry')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'telemetry' },
						}
					: {}
			"
		>
			<TelemetryDisclosureCard
				v-model="current.telemetry"
				:hide-description="hideDescription"
				:title="reviewTitle('telemetry')"
				:variant="variant"
				v-bind="disclosureUpdateProps('telemetry')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('telemetry', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="telemetry"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('derivative_work')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'derivative-content' },
						}
					: {}
			"
		>
			<DerivativeDisclosureCard
				v-model="current.derivative"
				:hide-description="hideDescription"
				:title="reviewTitle('derivative-content')"
				:variant="variant"
				v-bind="disclosureUpdateProps('derivative_work')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('derivative_work', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="derivative_work"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('epilepsy_triggers')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'photosensitivity' },
						}
					: {}
			"
		>
			<PhotosensitivityDisclosureCard
				v-model="current.photosensitivity"
				:hide-description="hideDescription"
				:title="reviewTitle('photosensitivity')"
				:variant="variant"
				v-bind="disclosureUpdateProps('epilepsy_triggers')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('epilepsy_triggers', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="epilepsy_triggers"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('system_interactions')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'system-interactions' },
						}
					: {}
			"
		>
			<SystemInteractionsDisclosureCard
				v-model="current.systemInteractions"
				:hide-description="hideDescription"
				:title="reviewTitle('system-interactions')"
				:variant="variant"
				v-bind="disclosureUpdateProps('system_interactions')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('system_interactions', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="system_interactions"
				class="mt-2"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('archived')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							target: { kind: 'disclosure', key: 'archive' },
						}
					: {}
			"
		>
			<ArchivedDisclosureCard
				v-model="current.archived"
				:project-title="projectTitle"
				:hide-description="hideDescription"
				:title="reviewTitle('archive')"
				:variant="variant"
				v-bind="disclosureUpdateProps('archived')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('archived', status)
				"
			/>
			<ProjectIssueCard
				v-if="variant === 'settings'"
				:target="disclosureIssueTargets"
				disclosure-type="archived"
				class="mt-2"
			/>
		</component>
	</div>
</template>
