<script setup lang="ts">
import ReviewPanel from '~/components/project-review/review-panel/index.vue'

import AiDisclosureCard from './AiDisclosureCard.vue'
import AiFunctionalityDisclosureCard from './AiFunctionalityDisclosureCard.vue'
import AdvertisingDisclosureCard from './AdvertisingDisclosureCard.vue'
import PaidFeaturesDisclosureCard from './PaidFeaturesDisclosureCard.vue'
import TelemetryDisclosureCard from './TelemetryDisclosureCard.vue'
import DerivativeDisclosureCard from './DerivativeDisclosureCard.vue'
import PhotosensitivityDisclosureCard from './PhotosensitivityDisclosureCard.vue'
import SystemInteractionsDisclosureCard from './SystemInteractionsDisclosureCard.vue'
import ArchivedDisclosureCard from './ArchivedDisclosureCard.vue'
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
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'ai' },
						}
					: {}
			"
		>
			<AiDisclosureCard
				v-model="current.ai"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('ai_content')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('ai_content', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('ai_functionality')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'ai-functionality' },
						}
					: {}
			"
		>
			<AiFunctionalityDisclosureCard
				v-model="current.aiFunctionality"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('ai_functionality')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('ai_functionality', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('advertisements')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'ads' },
						}
					: {}
			"
		>
			<AdvertisingDisclosureCard
				v-model="current.advertising"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('advertisements')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('advertisements', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('paid_features')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'paid-features' },
						}
					: {}
			"
		>
			<PaidFeaturesDisclosureCard
				v-model="current.paidFeatures"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('paid_features')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('paid_features', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('telemetry')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'telemetry' },
						}
					: {}
			"
		>
			<TelemetryDisclosureCard
				v-model="current.telemetry"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('telemetry')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('telemetry', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('derivative_work')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'derivative-content' },
						}
					: {}
			"
		>
			<DerivativeDisclosureCard
				v-model="current.derivative"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('derivative_work')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('derivative_work', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('epilepsy_triggers')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'photosensitivity' },
						}
					: {}
			"
		>
			<PhotosensitivityDisclosureCard
				v-model="current.photosensitivity"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('epilepsy_triggers')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('epilepsy_triggers', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('system_interactions')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'system-interactions' },
						}
					: {}
			"
		>
			<SystemInteractionsDisclosureCard
				v-model="current.systemInteractions"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('system_interactions')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('system_interactions', status)
				"
			/>
		</component>
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-if="isDisclosureVisible('archived')"
			v-bind="
				variant === 'review'
					? {
							mode: 'anchored',
							triggerPlacement: 'above',
							target: { kind: 'disclosure', key: 'archive' },
						}
					: {}
			"
		>
			<ArchivedDisclosureCard
				v-model="current.archived"
				:project-title="projectTitle"
				:hide-description="hideDescription"
				:variant="variant"
				v-bind="disclosureUpdateProps('archived')"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus('archived', status)
				"
			/>
		</component>
	</div>
</template>
