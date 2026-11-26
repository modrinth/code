<script setup lang="ts">
import { defineMessages, UnsavedChangesPopup } from '@modrinth/ui'
import { computed, shallowRef, watch } from 'vue'

import {
	inheritedRevenueWeight,
	isValidPayoutWeight,
	maxInheritedRevenueShare,
	revenueAllocation,
	revenueMembersWithOverrides,
	revenueWeightUpdates,
	withInheritedRevenueShares,
} from './revenue'
import {
	fullAllocation,
	type MemberWeight,
	toAllocationUnits,
} from './revenue-split-editor/allocation'
import RevenueSplitEditor from './revenue-split-editor/index.vue'
import type { MemberDraft } from './types'

const props = defineProps<{
	members: MemberDraft[]
	canToggleOverride?: (member: MemberDraft) => boolean
	canEdit: boolean
	busy: boolean
	saving: boolean
}>()
const emit = defineEmits<{
	save: [
		allocation: MemberWeight[],
		original: MemberDraft[],
		overrides: ReadonlyMap<string, boolean>,
	]
}>()

const messages = defineMessages({
	incompleteSplit: {
		id: 'project.settings.members.revenue.split-total-required',
		defaultMessage: 'The revenue split must total 100%.',
	},
	weightLimit: {
		id: 'project.settings.members.revenue.split-weight-limit',
		defaultMessage: 'This split exceeds the payout weight limit. Adjust the shares before saving.',
	},
})

const original = shallowRef<MemberDraft[]>([])
const draft = shallowRef<MemberWeight[]>([])
const overrides = shallowRef<ReadonlyMap<string, boolean>>(new Map())
const effectiveMembers = computed(() =>
	revenueMembersWithOverrides(original.value, overrides.value),
)
const initial = computed(() => revenueAllocation(effectiveMembers.value, { editing: true }))
const inheritedWeight = computed(() => inheritedRevenueWeight(effectiveMembers.value))
const readOnly = computed(
	() =>
		!props.canEdit ||
		(!props.members.some((member) => props.canToggleOverride?.(member)) &&
			(props.members.length === 1 || props.members.every((member) => member.override === false))),
)
const preview = computed(() => withInheritedRevenueShares(effectiveMembers.value, draft.value))
const allocation = computed({
	get: () => (readOnly.value ? revenueAllocation(props.members) : preview.value),
	set: (rows) => {
		const current = new Map(allocation.value.map(({ id, percentage }) => [id, percentage]))
		const requested = new Map(rows.map(({ id, percentage }) => [id, percentage]))
		draft.value = draft.value.map((row) => {
			const percentage = requested.get(row.id)
			return !row.fixed && percentage !== undefined && percentage !== current.get(row.id)
				? { ...row, percentage, shareEdited: inheritedWeight.value > 0 }
				: row
		})
		if (!sharesChanged.value) draft.value = initial.value
	},
})
const originalShares = computed(() => initial.value.map(({ percentage }) => percentage))
const draftShares = computed(() => preview.value.map(({ percentage }) => percentage))
const sharesChanged = computed(() =>
	draftShares.value.some((percentage, index) => percentage !== originalShares.value[index]),
)
const changed = computed(() => sharesChanged.value || overrides.value.size > 0)
const error = computed(() => {
	if (!sharesChanged.value) return undefined
	const total = allocation.value.reduce((sum, row) => sum + toAllocationUnits(row.percentage), 0)
	if (total !== fullAllocation) return messages.incompleteSplit
	if (
		revenueWeightUpdates(effectiveMembers.value, draft.value).some(
			({ weight }) => !isValidPayoutWeight(weight),
		)
	) {
		return messages.weightLimit
	}
	return undefined
})

function reset() {
	original.value = props.members
	overrides.value = new Map()
	draft.value = initial.value
}

function inheritedShareLimit(userId: string) {
	return maxInheritedRevenueShare(effectiveMembers.value, draft.value, userId)
}

function canToggleMemberOverride(userId: string) {
	const member = original.value.find((member) => member.user.id === userId)
	return !!member && (overrides.value.has(userId) || props.canToggleOverride?.(member) === true)
}

function setOverride(userId: string, enabled: boolean) {
	const member = original.value.find((member) => member.user.id === userId)
	if (props.busy || !member?.organizationDefaults || !canToggleMemberOverride(userId)) return

	const edited = new Map(
		draft.value
			.filter(
				(row, index) =>
					row.id !== userId &&
					(row.shareEdited || row.percentage !== initial.value[index]?.percentage),
			)
			.map((row) => [row.id, row]),
	)
	const next = new Map(overrides.value)
	if (enabled === member.override) next.delete(userId)
	else next.set(userId, enabled)
	overrides.value = next
	draft.value = initial.value.map((row) => {
		const edit = edited.get(row.id)
		return !row.fixed && edit
			? { ...row, percentage: edit.percentage, shareEdited: edit.shareEdited }
			: row
	})
}

watch(
	() => props.members,
	() => {
		if (!changed.value) reset()
	},
	{ immediate: true },
)

function save() {
	if (readOnly.value || props.busy || !changed.value || error.value) return
	emit('save', draft.value, original.value, overrides.value)
}

defineExpose({ reset })
</script>

<template>
	<div class="flex flex-col gap-3">
		<RevenueSplitEditor
			v-model="allocation"
			:read-only="readOnly"
			:disabled="busy"
			:share-limit="inheritedWeight > 0 ? inheritedShareLimit : undefined"
			:can-toggle-override="canToggleMemberOverride"
			@toggle-override="setOverride"
		/>
		<UnsavedChangesPopup
			v-if="!readOnly"
			:original="{ shares: originalShares, overrides: 0 }"
			:modified="{ shares: draftShares, overrides: overrides.size }"
			:saving="saving"
			:can-save="!busy && !error"
			:save-disabled-reason="error"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>
