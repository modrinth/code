<template>
	<NewModal
		ref="modal"
		:header="modalTitle"
		max-width="calc(100vw - 2rem)"
		width="720px"
		:on-hide="resetDraft"
	>
		<div class="flex flex-col gap-5">
			<label class="flex flex-col gap-2 font-semibold text-primary">
				{{ formatMessage(analyticsMessages.breakdownGroupsGroupName) }}
				<StyledInput
					v-model="draftName"
					:placeholder="formatMessage(analyticsMessages.breakdownGroupsGroupNamePlaceholder)"
				/>
			</label>

			<div class="flex flex-col gap-3">
				<div
					v-for="(series, seriesIndex) in draftSeries"
					:key="series.id"
					class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
				>
					<div class="flex items-end gap-2">
						<label class="flex min-w-0 flex-1 flex-col gap-2 font-semibold text-primary">
							{{ formatMessage(analyticsMessages.breakdownGroupsSeriesName) }}
							<StyledInput v-model="series.name" />
						</label>
						<ButtonStyled circular type="transparent">
							<button
								type="button"
								:aria-label="formatMessage(analyticsMessages.breakdownGroupsRemoveSeries)"
								@click="removeSeries(seriesIndex)"
							>
								<TrashIcon />
							</button>
						</ButtonStyled>
					</div>
					<label class="flex flex-col gap-2 font-semibold text-primary">
						{{ formatMessage(analyticsMessages.breakdownGroupsSeriesValues) }}
						<MultiSelect
							v-model="series.values"
							:options="getSeriesOptions(series.id)"
							:placeholder="formatMessage(analyticsMessages.breakdownGroupsSelectValues)"
							searchable
							fuzzy-search
							:clearable="false"
							checkbox-position="right"
							:max-tag-rows="2"
						>
							<template #input-content="{ isOpen, selectedOptions }">
								<div class="flex min-h-8 min-w-0 flex-1 items-center gap-2">
									<span
										class="min-w-0 flex-1 truncate font-medium"
										:class="selectedOptions.length === 0 ? 'text-secondary' : 'text-primary'"
									>
										{{
											selectedOptions.length === 0
												? formatMessage(analyticsMessages.breakdownGroupsSelectValues)
												: selectedOptions.map((option) => option.label).join(', ')
										}}
									</span>
									<ChevronLeftIcon
										class="size-5 shrink-0 text-secondary transition-transform duration-150"
										:class="isOpen ? 'rotate-90' : '-rotate-90'"
									/>
								</div>
							</template>
						</MultiSelect>
					</label>
				</div>

				<ButtonStyled type="transparent" class="w-fit">
					<button type="button" @click="addSeries">
						<PlusIcon />
						{{ formatMessage(analyticsMessages.breakdownGroupsAddSeries) }}
					</button>
				</ButtonStyled>
			</div>

			<div class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-4">
				<div class="font-semibold text-contrast">{{ formatMessage(analyticsMessages.other) }}</div>
				<div class="mt-1 text-sm text-secondary">
					{{
						formatMessage(analyticsMessages.breakdownGroupsOtherDescription, {
							count: unassignedValueCount,
						})
					}}
				</div>
			</div>

			<p v-if="validationMessage" class="m-0 text-sm font-semibold text-red">
				{{ validationMessage }}
			</p>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="transparent">
					<button type="button" @click="modal?.hide()">
						{{ formatMessage(analyticsMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" :disabled="Boolean(validationMessage)" @click="save">
						{{ formatMessage(analyticsMessages.saveButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronLeftIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	MultiSelect,
	type MultiSelectOption,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

import { createAnalyticsBreakdownGroupId } from '~/components/analytics-dashboard/breakdown-groups'
import {
	type AnalyticsBreakdownGroup,
	type AnalyticsBreakdownGroupSeries,
	type AnalyticsBreakdownPreset,
	doesAnalyticsPointMatchNormalizedFilters,
	getProjectIdsMatchingStatusFilter,
	injectAnalyticsDashboardContext,
	normalizeAnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

import { formatBreakdownLabel } from '../analytics-chart/analytics-chart-utils'
import { analyticsMessages } from '../analytics-messages'
import {
	ALL_BREAKDOWN_VALUE,
	getAnalyticsBreakdownValue,
	isNoDependentAnalyticsBreakdownValue,
	isUnknownAnalyticsBreakdownValue,
} from '../breakdown'

type Breakdown = Exclude<AnalyticsBreakdownPreset, 'none'>

const emit = defineEmits<{
	save: [group: AnalyticsBreakdownGroup, activate: boolean]
}>()

const {
	breakdownGroups,
	displayedTimeSlices,
	displayedSelectedProjectIds,
	displayedSelectedFilters,
	projectStatusById,
	dependentProjectTypesById,
	projectNamesById,
	userNamesById,
	getVersionDisplayName,
} = injectAnalyticsDashboardContext()
const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const editingGroupId = ref<string | null>(null)
const draftBreakdown = ref<Breakdown>('country')
const draftName = ref('')
const draftSeries = ref<AnalyticsBreakdownGroupSeries[]>([])

const modalTitle = computed(() =>
	formatMessage(
		editingGroupId.value
			? analyticsMessages.breakdownGroupsModalEditTitle
			: analyticsMessages.breakdownGroupsModalCreateTitle,
	),
)

const availableValueSet = computed(
	() => new Set(availableValues.value.map((option) => option.value)),
)
const availableValues = computed<MultiSelectOption<string>[]>(() => {
	const projectIds = new Set(
		getProjectIdsMatchingStatusFilter(
			displayedSelectedProjectIds.value,
			projectStatusById.value,
			displayedSelectedFilters.value,
		),
	)
	const normalizedFilters = normalizeAnalyticsSelectedFilters(displayedSelectedFilters.value)
	const values = new Set<string>()
	for (const slice of displayedTimeSlices.value) {
		for (const point of slice) {
			if (!isProjectAnalyticsPoint(point) || !projectIds.has(point.source_project)) continue
			if (
				!doesAnalyticsPointMatchNormalizedFilters(
					point,
					normalizedFilters,
					dependentProjectTypesById.value,
				)
			) {
				continue
			}
			const value = getAnalyticsBreakdownValue(point, draftBreakdown.value, formatMessage)
			if (value !== ALL_BREAKDOWN_VALUE) values.add(value)
		}
	}
	return [...values]
		.map((value) => ({ value, label: getValueLabel(value) }))
		.sort((left, right) => left.label.localeCompare(right.label))
})

const allDraftValues = computed(() => draftSeries.value.flatMap((series) => series.values))
const unassignedValueCount = computed(() => {
	const assignedValues = new Set(allDraftValues.value)
	return availableValues.value.filter((option) => !assignedValues.has(option.value)).length
})

const validationMessage = computed(() => {
	const normalizedName = draftName.value.trim().toLocaleLowerCase()
	if (!normalizedName) return formatMessage(analyticsMessages.breakdownGroupsNameRequired)
	if (
		breakdownGroups.value.some(
			(group) =>
				group.breakdown === draftBreakdown.value &&
				group.id !== editingGroupId.value &&
				group.name.trim().toLocaleLowerCase() === normalizedName,
		)
	) {
		return formatMessage(analyticsMessages.breakdownGroupsNameDuplicate)
	}
	if (draftSeries.value.length === 0) {
		return formatMessage(analyticsMessages.breakdownGroupsSeriesRequired)
	}
	const names = draftSeries.value.map((series) => series.name.trim().toLocaleLowerCase())
	if (names.some((name) => !name)) {
		return formatMessage(analyticsMessages.breakdownGroupsSeriesNameRequired)
	}
	if (new Set(names).size !== names.length) {
		return formatMessage(analyticsMessages.breakdownGroupsSeriesNameDuplicate)
	}
	if (draftSeries.value.some((series) => series.values.length === 0)) {
		return formatMessage(analyticsMessages.breakdownGroupsSeriesValuesRequired)
	}
	return ''
})

function isProjectAnalyticsPoint(
	point: Labrinth.Analytics.v3.AnalyticsData,
): point is Labrinth.Analytics.v3.ProjectAnalytics {
	return 'source_project' in point
}

function getValueLabel(value: string) {
	if (draftBreakdown.value === 'project' || draftBreakdown.value === 'dependent_project_download') {
		if (isNoDependentAnalyticsBreakdownValue(value)) {
			return formatMessage(analyticsMessages.noDependent)
		}
		if (!isUnknownAnalyticsBreakdownValue(value)) {
			return projectNamesById.value.get(value) ?? value
		}
	}
	return formatBreakdownLabel(
		value,
		draftBreakdown.value,
		getVersionDisplayName,
		userNamesById.value,
		formatMessage,
	)
}

function getSeriesOptions(seriesId: string): MultiSelectOption<string>[] {
	const selectedByOtherSeries = new Set(
		draftSeries.value.filter((series) => series.id !== seriesId).flatMap((series) => series.values),
	)
	const currentSeries = draftSeries.value.find((series) => series.id === seriesId)
	const unavailableValues = (currentSeries?.values ?? []).filter(
		(value) => !availableValueSet.value.has(value),
	)
	return [
		...availableValues.value.map((option) => ({
			...option,
			disabled: selectedByOtherSeries.has(option.value),
		})),
		...unavailableValues.map((value) => ({
			value,
			label: formatMessage(analyticsMessages.breakdownGroupsUnavailableValue, {
				value: getValueLabel(value),
			}),
		})),
	]
}

function createEmptySeries(): AnalyticsBreakdownGroupSeries {
	return { id: createAnalyticsBreakdownGroupId(), name: '', values: [] }
}

function addSeries() {
	draftSeries.value.push(createEmptySeries())
}

function removeSeries(index: number) {
	draftSeries.value.splice(index, 1)
}

function show(breakdown: Breakdown, group?: AnalyticsBreakdownGroup, event?: MouseEvent) {
	draftBreakdown.value = breakdown
	editingGroupId.value = group?.id ?? null
	draftName.value = group?.name ?? ''
	draftSeries.value = group
		? group.series.map((series) => ({ ...series, values: [...series.values] }))
		: [createEmptySeries()]
	modal.value?.show(event)
}

function resetDraft() {
	editingGroupId.value = null
	draftName.value = ''
	draftSeries.value = []
}

function save() {
	if (validationMessage.value) return
	const isNewGroup = editingGroupId.value === null
	emit(
		'save',
		{
			id: editingGroupId.value ?? createAnalyticsBreakdownGroupId(),
			name: draftName.value.trim(),
			breakdown: draftBreakdown.value,
			series: draftSeries.value.map((series) => ({
				id: series.id,
				name: series.name.trim(),
				values: [...new Set(series.values)],
			})),
		},
		isNewGroup,
	)
	modal.value?.hide()
}

defineExpose({ show })
</script>
