<script setup lang="ts">
import { OrganizationIcon } from '@modrinth/assets'
import { defineMessages, Input, Toggle, useVIntl } from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import { revenuePercentMessage } from '../revenue'
import { fullAllocation, type MemberWeight, toAllocationUnits } from './allocation'

const members = defineModel<MemberWeight[]>({ required: true })
const props = defineProps<{
	readOnly?: boolean
	disabled?: boolean
	shareLimit?: (userId: string) => number
	canToggleOverride?: (userId: string) => boolean
}>()
const emit = defineEmits<{ toggleOverride: [userId: string, enabled: boolean] }>()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	share: {
		id: 'project.settings.members.weights.share',
		defaultMessage: '{name}’s revenue share',
	},
	memberPercent: {
		id: 'project.settings.members.weights.member-percent',
		defaultMessage: '{name} - {share, number, ::percent .##}',
	},
	organizationMember: {
		id: 'project.settings.members.weights.organization-member',
		defaultMessage: 'Organization member',
	},
	unallocated: {
		id: 'project.settings.members.weights.unallocated',
		defaultMessage: '{share, number, ::percent .##} unallocated',
	},
	override: {
		id: 'project.settings.members.revenue.override',
		defaultMessage: 'Override',
	},
	overrideMember: {
		id: 'project.settings.members.revenue.override-member',
		defaultMessage: 'Override revenue share for {name}',
	},
})

const hasOrganizationMembers = computed(() =>
	members.value.some((member) => member.organizationMember),
)
const allocatedMembers = computed(() => members.value.filter((row) => row.percentage > 0))
const remaining = computed(
	() =>
		fullAllocation -
		members.value.reduce((total, row) => total + toAllocationUnits(row.percentage), 0),
)
const inputDraft = ref<{ id: string; value: string }>()
const sliderDraft = ref<{ id: string; value: number }>()
const memberList = useTemplateRef('memberList')

function maxShare(member: MemberWeight) {
	if (props.shareLimit) return props.shareLimit(member.id)
	const current = toAllocationUnits(member.percentage)
	return Math.max(0, current + remaining.value) / 100
}

function isFixed(member: MemberWeight) {
	return member.fixed || members.value.length === 1
}

function updateShare(member: MemberWeight, requested: number) {
	if (props.readOnly || props.disabled || isFixed(member) || !Number.isFinite(requested)) return
	const percentage = toAllocationUnits(Math.min(maxShare(member), Math.max(0, requested))) / 100
	if (percentage === member.percentage) return
	members.value = members.value.map((row) => (row.id === member.id ? { ...row, percentage } : row))
}

function parseInput(value: string) {
	return value.trim() ? Number(value.replace(',', '.')) : NaN
}

function typeShare(member: MemberWeight, value: string | number | undefined) {
	sliderDraft.value = undefined
	const text = String(value ?? '')
	inputDraft.value = { id: member.id, value: text }
	const share = parseInput(text)
	if (share >= 0 && share <= maxShare(member)) updateShare(member, share)
}

function commitShare(member: MemberWeight) {
	if (inputDraft.value?.id !== member.id) return
	updateShare(member, parseInput(inputDraft.value.value))
	inputDraft.value = undefined
}

function stepShare(member: MemberWeight, step: number) {
	inputDraft.value = undefined
	sliderDraft.value = undefined
	const percentage = step > 0 ? Math.floor(member.percentage) : Math.ceil(member.percentage)
	updateShare(member, percentage + step)
}

function focusAdjacentShare(event: KeyboardEvent) {
	if (event.altKey || event.ctrlKey || event.metaKey || !memberList.value) return
	const inputs = Array.from(
		memberList.value.querySelectorAll<HTMLInputElement>('input[data-share-input]:enabled'),
	)
	const index = inputs.indexOf(event.currentTarget as HTMLInputElement)
	if (index === -1) return
	const next = inputs[index + (event.shiftKey ? -1 : 1)]
	if (!next) return
	event.preventDefault()
	next.focus()
}

function exceedsMaximum(member: MemberWeight) {
	return inputDraft.value?.id === member.id && parseInput(inputDraft.value.value) > maxShare(member)
}

function sliderExceedsMaximum(member: MemberWeight) {
	return sliderDraft.value?.id === member.id && sliderDraft.value.value > maxShare(member)
}

function slideShare(member: MemberWeight, event: Event) {
	const input = event.currentTarget as HTMLInputElement
	const requested = Math.round(input.valueAsNumber)
	inputDraft.value = undefined
	sliderDraft.value = { id: member.id, value: requested }
	updateShare(member, requested)
	input.value = String(Math.min(maxShare(member), requested))
}
</script>

<template>
	<div
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4 sm:p-5"
	>
		<div class="flex h-3 overflow-hidden rounded-full bg-surface-3">
			<div
				v-for="member in allocatedMembers"
				:key="member.id"
				v-tooltip="
					formatMessage(messages.memberPercent, {
						name: member.name,
						share: member.percentage / 100,
					})
				"
				role="img"
				:aria-label="
					formatMessage(messages.memberPercent, {
						name: member.name,
						share: member.percentage / 100,
					})
				"
				tabindex="0"
				class="shrink-0 focus-visible:-outline-offset-2"
				:style="{ width: `${member.percentage}%`, backgroundColor: member.color }"
			/>
			<div
				v-if="remaining > 0"
				v-tooltip="formatMessage(messages.unallocated, { share: remaining / fullAllocation })"
				:aria-label="formatMessage(messages.unallocated, { share: remaining / fullAllocation })"
				role="img"
				tabindex="0"
				class="unallocated-share min-w-0 flex-1 focus-visible:-outline-offset-2"
			/>
		</div>
		<ul
			ref="memberList"
			class="m-0 grid list-none grid-cols-[minmax(0,1fr)_max-content] gap-x-3 gap-y-3 p-0"
			:class="{
				'sm:grid-cols-[minmax(0,1fr)_auto_33.333%_max-content]':
					!readOnly && hasOrganizationMembers,
				'sm:grid-cols-[minmax(0,1fr)_33.333%_max-content]': !readOnly && !hasOrganizationMembers,
			}"
		>
			<li
				v-for="member in members"
				:key="member.id"
				class="col-span-full grid grid-cols-subgrid items-center gap-y-1"
			>
				<div class="flex min-w-0 items-center gap-2">
					<span class="size-3 shrink-0 rounded-full" :style="{ backgroundColor: member.color }" />
					<span class="min-w-0 break-words font-medium text-contrast">{{ member.name }}</span>
					<OrganizationIcon
						v-if="member.organizationMember"
						v-tooltip="formatMessage(messages.organizationMember)"
						role="img"
						:aria-label="formatMessage(messages.organizationMember)"
						class="size-4 shrink-0 text-secondary"
					/>
				</div>
				<div v-if="!readOnly" class="col-span-full row-start-2 flex items-center gap-3 sm:contents">
					<div v-if="hasOrganizationMembers" class="w-12 shrink-0">
						<Toggle
							v-if="member.organizationMember"
							v-tooltip="formatMessage(messages.override)"
							:model-value="!member.fixed"
							:disabled="disabled || !canToggleOverride?.(member.id)"
							:aria-label="formatMessage(messages.overrideMember, { name: member.name })"
							@update:model-value="emit('toggleOverride', member.id, $event)"
						/>
					</div>
					<input
						type="range"
						min="0"
						max="100"
						step="any"
						:value="member.percentage"
						:disabled="disabled || isFixed(member)"
						:aria-label="formatMessage(messages.share, { name: member.name })"
						:aria-valuetext="
							formatMessage(revenuePercentMessage, { share: member.percentage / 100 })
						"
						:style="{
							'--slider-color': sliderExceedsMaximum(member) ? 'var(--color-red)' : member.color,
							'--slider-fill': `${member.percentage}%`,
						}"
						class="share-slider h-6 w-1/3 min-w-0 cursor-pointer appearance-none !border-0 !bg-transparent !p-0 !shadow-none focus-visible:rounded-md focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand disabled:cursor-not-allowed disabled:opacity-50 sm:w-full"
						@input="slideShare(member, $event)"
						@change="sliderDraft = undefined"
						@blur="sliderDraft = undefined"
						@pointerup="sliderDraft = undefined"
						@pointercancel="sliderDraft = undefined"
					/>
				</div>
				<span v-if="readOnly" class="whitespace-nowrap text-right tabular-nums text-secondary">
					{{ formatMessage(revenuePercentMessage, { share: member.percentage / 100 }) }}
				</span>
				<div v-else class="flex items-center justify-end gap-2">
					<Input
						:model-value="
							inputDraft?.id === member.id ? inputDraft.value : String(member.percentage)
						"
						:disabled="disabled || isFixed(member)"
						:error="exceedsMaximum(member) || sliderExceedsMaximum(member)"
						:aria-label="formatMessage(messages.share, { name: member.name })"
						inputmode="decimal"
						data-share-input
						input-class="text-right tabular-nums"
						size="small"
						class="w-24"
						:class="{
							'focus-within:!ring-highlight-red':
								exceedsMaximum(member) || sliderExceedsMaximum(member),
						}"
						@update:model-value="typeShare(member, $event)"
						@focus="($event.currentTarget as HTMLInputElement).select()"
						@keydown.enter.prevent="commitShare(member)"
						@keydown.up.prevent="stepShare(member, 1)"
						@keydown.down.prevent="stepShare(member, -1)"
						@keydown.tab="focusAdjacentShare"
						@blur="commitShare(member)"
					>
						<template #trailing><span aria-hidden="true" class="text-secondary">%</span></template>
					</Input>
				</div>
			</li>
		</ul>
	</div>
</template>

<style scoped lang="scss">
.unallocated-share {
	background: repeating-linear-gradient(135deg, var(--surface-3) 0 4px, var(--surface-5) 4px 6px);
}

.share-slider {
	@mixin track {
		height: 0.375rem;
		border-radius: 9999px;
		background: linear-gradient(
			to right,
			var(--slider-color) var(--slider-fill),
			var(--surface-5) var(--slider-fill)
		);
	}

	@mixin thumb {
		width: 0.875rem;
		height: 0.875rem;
		border: 0;
		border-radius: 50%;
		background: var(--slider-color);
	}

	&::-webkit-slider-runnable-track {
		@include track;
	}

	&::-moz-range-track {
		@include track;
	}

	&::-webkit-slider-thumb {
		@include thumb;
		appearance: none;
		margin-top: -0.25rem;
	}

	&::-moz-range-thumb {
		@include thumb;
	}
}
</style>
