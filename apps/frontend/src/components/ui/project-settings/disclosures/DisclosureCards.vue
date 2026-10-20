<script setup lang="ts">
import { computed, ref } from 'vue'

import DisclosureView from '~/components/project-review/disclosures/view.vue'
import ReviewPanel from '~/components/project-review/review-panel/index.vue'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import AdvertisingDisclosureCard from './AdvertisingDisclosureCard.vue'
import AiDisclosureCard from './AiDisclosureCard.vue'
import AiFunctionalityDisclosureCard from './AiFunctionalityDisclosureCard.vue'
import ArchivedDisclosureCard from './ArchivedDisclosureCard.vue'
import DerivativeDisclosureCard from './DerivativeDisclosureCard.vue'
import { provideDisclosureDraft } from './draft-context'
import PaidFeaturesDisclosureCard from './PaidFeaturesDisclosureCard.vue'
import PhotosensitivityDisclosureCard from './PhotosensitivityDisclosureCard.vue'
import SystemInteractionsDisclosureCard from './SystemInteractionsDisclosureCard.vue'
import TelemetryDisclosureCard from './TelemetryDisclosureCard.vue'
import type { DisclosureLockStatus, DisclosureType } from './types'
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
const emit = defineEmits<{ editingChange: [editing: boolean] }>()

const { current, disclosureUpdateProps, setDisclosureLockStatus, isDisclosureVisible } =
	props.editor
provideDisclosureDraft(current)
const panels = injectReviewPanels(null)
const editingTypes = ref(new Set<DisclosureType>())

function toggleEditing(type: DisclosureType) {
	const next = new Set(editingTypes.value)
	if (next.has(type)) next.delete(type)
	else next.add(type)
	editingTypes.value = next
	emit('editingChange', next.size > 0)
}

function cancelEditing(type: DisclosureType) {
	props.editor.resetDisclosure(type, true)
	toggleEditing(type)
}

async function saveEditing(type: DisclosureType) {
	if (await props.editor.saveDisclosure(type)) toggleEditing(type)
}

function cardProps(type: DisclosureType) {
	return {
		...disclosureUpdateProps(type),
		...(props.variant === 'review'
			? {
					editing: editingTypes.value.has(type),
					onToggleEditing: () => toggleEditing(type),
					onCancelEditing: () => cancelEditing(type),
					onSaveEditing: () => saveEditing(type),
					sectionCanSave: props.editor.canSaveDisclosure(type),
					sectionSaveDisabledReason: props.editor.disclosureSaveDisabledReason(type),
					sectionSaveError: props.editor.disclosureSaveError(type),
					sectionSaving: props.editor.saving.value,
				}
			: {}),
	}
}

function reviewTitle(key: string): string | undefined {
	return props.variant === 'review'
		? panels?.resolve({ kind: 'disclosure', key })?.panel.title
		: undefined
}

const cards = computed(() => {
	const activeTypes = new Set(props.editor.savedSnapshot.value.disclosures.map(({ type }) => type))
	const visible = [
		{ type: 'ai_content', key: 'ai', enabled: activeTypes.has('ai_content') },
		{
			type: 'ai_functionality',
			key: 'ai-functionality',
			enabled: activeTypes.has('ai_functionality'),
		},
		{
			type: 'advertisements',
			key: 'ads',
			enabled: activeTypes.has('advertisements'),
		},
		{
			type: 'paid_features',
			key: 'paid-features',
			enabled: activeTypes.has('paid_features'),
		},
		{
			type: 'telemetry',
			key: 'telemetry',
			enabled: activeTypes.has('telemetry'),
		},
		{
			type: 'derivative_work',
			key: 'derivative-content',
			enabled: activeTypes.has('derivative_work'),
		},
		{
			type: 'epilepsy_triggers',
			key: 'photosensitivity',
			enabled: activeTypes.has('epilepsy_triggers'),
		},
		{
			type: 'system_interactions',
			key: 'system-interactions',
			enabled: activeTypes.has('system_interactions'),
		},
		{
			type: 'archived',
			key: 'archive',
			enabled: activeTypes.has('archived'),
		},
	] satisfies { type: DisclosureType; key: string; enabled: boolean }[]
	const filtered = visible.filter(({ type }) => isDisclosureVisible(type))
	return props.variant === 'review'
		? filtered.sort((a, b) => Number(b.enabled) - Number(a.enabled))
		: filtered
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<component
			:is="variant === 'review' ? ReviewPanel : 'div'"
			v-for="card in cards"
			:key="card.type"
			v-bind="
				variant === 'review'
					? { mode: 'anchored', target: { kind: 'disclosure', key: card.key } }
					: {}
			"
		>
			<DisclosureView
				v-if="variant === 'review' && !editingTypes.has(card.type)"
				:type="card.type"
				@edit="toggleEditing(card.type)"
			/>
			<AiDisclosureCard
				v-else-if="card.type === 'ai_content'"
				v-model="current.ai"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<AiFunctionalityDisclosureCard
				v-else-if="card.type === 'ai_functionality'"
				v-model="current.aiFunctionality"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<AdvertisingDisclosureCard
				v-else-if="card.type === 'advertisements'"
				v-model="current.advertising"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<PaidFeaturesDisclosureCard
				v-else-if="card.type === 'paid_features'"
				v-model="current.paidFeatures"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<TelemetryDisclosureCard
				v-else-if="card.type === 'telemetry'"
				v-model="current.telemetry"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<DerivativeDisclosureCard
				v-else-if="card.type === 'derivative_work'"
				v-model="current.derivative"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<PhotosensitivityDisclosureCard
				v-else-if="card.type === 'epilepsy_triggers'"
				v-model="current.photosensitivity"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<SystemInteractionsDisclosureCard
				v-else-if="card.type === 'system_interactions'"
				v-model="current.systemInteractions"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
			<ArchivedDisclosureCard
				v-else-if="card.type === 'archived'"
				v-model="current.archived"
				:project-title="projectTitle"
				:hide-description="hideDescription"
				:title="reviewTitle(card.key)"
				:variant="variant"
				v-bind="cardProps(card.type)"
				@set-lock-status="
					(status: DisclosureLockStatus) => setDisclosureLockStatus(card.type, status)
				"
			/>
		</component>
	</div>
</template>
