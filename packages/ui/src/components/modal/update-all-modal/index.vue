<script setup lang="ts">
import {
	CircleAlertIcon,
	DownloadIcon,
	FileTextIcon,
	RightArrowIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { renderHighlightedString } from '@modrinth/utils'
import { AnimatePresence, Motion, useReducedMotion } from 'motion-v'
import { computed, nextTick, ref, useId, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { useVIntl } from '#ui/composables/i18n'
import { dismissTooltip } from '#ui/providers/tooltip'
import { commonMessages } from '#ui/utils/common-messages'
import { getModifiedSelection } from '#ui/utils/modified-selection'

import { messages } from './update-all-modal-messages'
import type { UpdateAllItem, UpdateAllSelection } from './update-all-modal-types'
import UpdateAllModalVersionSelect from './update-all-modal-version-select.vue'
import { useUpdateAllSelection } from './use-update-all-selection'

const props = withDefaults(
	defineProps<{
		items: UpdateAllItem[]
		loading?: boolean
		loadingChangelog?: boolean
		actionLoading?: boolean
		actionDisabled?: boolean
	}>(),
	{
		loading: false,
		loadingChangelog: false,
		actionLoading: false,
		actionDisabled: false,
	},
)

const emit = defineEmits<{
	update: [selections: UpdateAllSelection[]]
	cancel: []
	changelog: [selection: UpdateAllSelection]
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const tableContainer = ref<HTMLElement>()
const changelogItemId = ref<string>()
const changelogCloseButton = ref<InstanceType<typeof IconButton>>()
const changelogHeadingId = useId()
const isOpen = ref(false)
const selectionAnchorId = ref<string>()
const focusAfterChangelogExit = ref<string>()
const prefersReducedMotion = useReducedMotion()
let submitted = false

const { rows, selections, allSelected, indeterminate, selectItem, selectAll, selectVersion, reset } =
	useUpdateAllSelection(() => props.items)

const activeRow = computed(() =>
	props.loading
		? undefined
		: rows.value.find((row) => row.id === changelogItemId.value && row.version),
)
const controlsDisabled = computed(() => props.loading || props.actionLoading)
const updateDisabled = computed(
	() => controlsDisabled.value || props.actionDisabled || selections.value.length === 0,
)
const reservedUpdateLabelCounts = computed(() => [...new Set([0, 1, 2, rows.value.length])])

watch(
	() => [isOpen.value, activeRow.value?.id, activeRow.value?.version?.id] as const,
	([open]) => {
		const row = activeRow.value
		if (open && row?.version) {
			emit('changelog', { id: row.id, projectId: row.project.id, version: row.version })
		}
	},
)

async function openChangelog(id: string) {
	dismissTooltip()
	focusAfterChangelogExit.value = undefined
	changelogItemId.value = id
	await nextTick()
	if (activeRow.value?.id === id) changelogCloseButton.value?.element?.focus({ preventScroll: true })
}

function closeChangelog() {
	focusAfterChangelogExit.value = changelogItemId.value
	changelogItemId.value = undefined
}

function handleChangelogExitComplete() {
	const id = focusAfterChangelogExit.value
	if (!id || activeRow.value) return
	focusAfterChangelogExit.value = undefined
	Array.from(tableContainer.value?.querySelectorAll<HTMLElement>('[data-changelog-id]') ?? [])
		.find((button) => button.dataset.changelogId === id)
		?.focus()
}

function show(options?: { changelogItemId?: string }) {
	if (isOpen.value) return
	reset()
	selectionAnchorId.value = undefined
	focusAfterChangelogExit.value = undefined
	submitted = false
	changelogItemId.value = options?.changelogItemId
	isOpen.value = true
	modal.value?.show()
}

function hide() {
	if (!isOpen.value) return
	modal.value?.hide()
}

function handleHide() {
	isOpen.value = false
	if (!submitted) emit('cancel')
}

function handleSelectAll(selected: boolean) {
	selectAll(selected)
	selectionAnchorId.value = undefined
}

function handleModifiedSelection(id: string, event: MouseEvent): boolean {
	if (controlsDisabled.value || event.button !== 0) return false

	const selectableIds = rows.value.filter((row) => row.version).map((row) => row.id)
	const selectedIds = new Set(selections.value.map((selection) => selection.id))
	const next = getModifiedSelection(
		selectableIds,
		selectedIds,
		id,
		selectionAnchorId.value,
		event,
	)
	if (!next) return false

	for (const selectableId of selectableIds) {
		selectItem(selectableId, next.selectedIds.has(selectableId))
	}
	selectionAnchorId.value = next.anchorId
	event.preventDefault()
	return true
}

function handleCheckboxSelection(id: string, selected: boolean, event?: MouseEvent) {
	if (event && handleModifiedSelection(id, event)) return
	selectItem(id, selected)
	selectionAnchorId.value = id
}

function isInteractiveRowTarget(event: MouseEvent): boolean {
	return (
		event.target instanceof Element &&
		!!event.target.closest('button, a, input, select, textarea, [role="button"], [role="checkbox"]')
	)
}

function handleRowMouseDown(event: MouseEvent) {
	if (event.shiftKey && !isInteractiveRowTarget(event)) event.preventDefault()
}

function handleRowClick(id: string, event: MouseEvent) {
	if (event.button !== 0 || isInteractiveRowTarget(event)) return
	if (handleModifiedSelection(id, event)) return
	if (activeRow.value && !controlsDisabled.value && rows.value.some((row) => row.id === id && row.version)) {
		openChangelog(id)
	}
}

function handleProjectClick(id: string, event: MouseEvent) {
	if (!handleModifiedSelection(id, event)) openChangelog(id)
}

function update() {
	if (updateDisabled.value || submitted) return
	submitted = true
	const selected = selections.value.map((selection) => ({ ...selection }))
	hide()
	emit('update', selected)
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		width="960px"
		max-width="960px"
		class="!rounded-[20px] !bg-surface-3"
		no-padding
		:disable-close="actionLoading"
		:on-hide="handleHide"
	>
		<div class="relative flex h-[643px] max-h-[calc(100dvh-320px)] min-h-0 overflow-hidden bg-surface-2 md:max-h-[calc(100dvh-240px)]">
			<div
				ref="tableContainer"
				class="w-full min-w-0 overflow-auto motion-safe:transition-opacity motion-safe:duration-300 motion-safe:ease-out"
				:class="activeRow ? 'hidden opacity-60 md:block' : 'opacity-100'"
				:aria-busy="loading"
			>
				<table
					v-if="!loading && rows.length"
					class="w-full min-w-[900px] table-fixed border-collapse text-left"
					:aria-label="formatMessage(messages.header)"
				>
					<colgroup>
						<col class="w-[304px]" />
						<col />
						<col class="w-[128px]" />
					</colgroup>
					<thead class="sticky top-0 z-[1] bg-surface-3">
						<tr class="h-12 border-0 border-b border-solid border-surface-5">
							<th scope="col" class="px-4 py-3 font-semibold">
								<div class="flex items-center gap-3">
									<Checkbox
										class="[&>span:first-child]:!size-6 [&>span:first-child]:!rounded-lg"
										:model-value="allSelected"
										:indeterminate="indeterminate"
										:description="formatMessage(messages.selectAll)"
										:disabled="controlsDisabled || !rows.some((row) => row.version)"
										@update:model-value="handleSelectAll"
									/>
									{{ formatMessage(messages.project) }}
								</div>
						</th>
						<th
							scope="col"
							class="px-4 py-3 font-semibold"
							:inert="!!activeRow"
						>
							{{ formatMessage(messages.versions) }}
						</th>
						<th
							scope="col"
							class="px-4 py-3 text-right font-semibold"
							:inert="!!activeRow"
						>
							<span class="block truncate">{{ formatMessage(commonMessages.changelogLabel) }}</span>
						</th>
						</tr>
					</thead>
					<tbody>
						<tr
							v-for="(row, index) in rows"
							:key="row.id"
							class="h-[57px] border-0 border-b border-solid border-surface-5"
							:class="[
								index % 2 ? 'bg-surface-1.5' : 'bg-surface-2',
								row.version && !controlsDisabled ? 'transition-colors hover:bg-surface-3' : '',
								activeRow && row.version && !controlsDisabled ? 'cursor-pointer' : '',
							]"
							@mousedown="handleRowMouseDown"
							@click="handleRowClick(row.id, $event)"
						>
							<td class="px-4 py-3">
								<div class="flex min-w-0 items-center gap-3">
									<Checkbox
										class="shrink-0 [&>span:first-child]:!size-6 [&>span:first-child]:!rounded-lg"
										:model-value="row.selected"
										:description="formatMessage(messages.selectProject, { project: row.project.title })"
										:disabled="controlsDisabled || !row.version"
										@update:model-value="(selected, event) => handleCheckboxSelection(row.id, selected, event)"
									/>
									<div class="flex min-w-0 items-center gap-2">
										<Avatar :src="row.project.icon_url" size="28px" class="!rounded-lg" />
										<button
											v-if="activeRow"
											type="button"
											class="min-w-0 cursor-pointer truncate border-0 bg-transparent p-0 text-left font-medium text-contrast hover:underline"
											:disabled="!row.version || controlsDisabled"
											:aria-label="formatMessage(messages.viewChangelog, { project: row.project.title })"
											@click="handleProjectClick(row.id, $event)"
										>
											{{ row.project.title }}
										</button>
										<span v-else class="truncate font-medium text-contrast" :title="row.project.title">
											{{ row.project.title }}
										</span>
									</div>
								</div>
							</td>
							<td
								class="px-4 py-3"
								:inert="!!activeRow"
							>
								<div class="flex min-w-0 items-center gap-2">
									<span class="min-w-0 shrink truncate" :title="row.currentVersion.version_number">
										{{ row.currentVersion.version_number }}
									</span>
									<RightArrowIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
									<UpdateAllModalVersionSelect
										v-if="row.version"
										:versions="row.versions"
										:version="row.version"
										:label="formatMessage(messages.selectVersion, { project: row.project.title })"
										:disabled="controlsDisabled"
										@select="selectVersion(row.id, $event)"
									/>
									<span v-else class="text-sm text-secondary">{{ formatMessage(messages.noVersion) }}</span>
								</div>
							</td>
							<td
								class="px-4 py-3"
								:inert="!!activeRow"
							>
								<div class="flex justify-end">
									<IconButton
										v-tooltip="activeRow ? null : formatMessage(messages.viewChangelog, { project: row.project.title })"
										size="sm"
										:data-changelog-id="row.id"
										:label="formatMessage(messages.viewChangelog, { project: row.project.title })"
										:disabled="!row.version || controlsDisabled"
										@click="openChangelog(row.id)"
									>
										<FileTextIcon class="size-4" aria-hidden="true" />
									</IconButton>
								</div>
							</td>
						</tr>
					</tbody>
				</table>
				<div v-else class="flex h-full items-center justify-center gap-2 p-6 text-secondary" role="status">
					<SpinnerIcon v-if="loading" class="size-6 animate-spin" aria-hidden="true" />
					{{ formatMessage(loading ? messages.loading : messages.empty) }}
				</div>
			</div>

			<AnimatePresence :initial="false" :on-exit-complete="handleChangelogExitComplete">
				<Motion
					v-if="activeRow && activeRow.version"
					key="changelog-panel"
					as="section"
					class="absolute inset-y-0 right-0 flex w-full min-w-0 flex-col border-0 border-l border-solid border-surface-5 bg-surface-2 md:w-[calc(100%_-_304px)]"
					:aria-labelledby="changelogHeadingId"
					:initial="{ x: prefersReducedMotion ? 0 : '100%', opacity: 0 }"
					:animate="{ x: 0, opacity: 1 }"
					:exit="{ x: prefersReducedMotion ? 0 : '100%', opacity: 0 }"
					:transition="{ duration: 0.3, ease: 'easeOut' }"
				>
					<div class="flex min-h-[68px] flex-wrap items-center gap-2 border-0 border-b border-solid border-surface-5 p-4">
						<Avatar :src="activeRow.project.icon_url" size="36px" class="!rounded-lg" />
						<h3 :id="changelogHeadingId" class="m-0 min-w-0 flex-1 truncate text-xl font-semibold text-contrast">
							{{ activeRow.project.title }}
						</h3>
						<Checkbox
							class="[&>span:first-child]:!size-6 [&>span:first-child]:!rounded-lg"
							:model-value="activeRow.selected"
							:description="formatMessage(messages.selectProject, { project: activeRow.project.title })"
							:disabled="controlsDisabled"
							@update:model-value="(selected, event) => handleCheckboxSelection(activeRow.id, selected, event)"
						/>
						<UpdateAllModalVersionSelect
							:versions="activeRow.versions"
							:version="activeRow.version"
							:label="formatMessage(messages.selectVersion, { project: activeRow.project.title })"
							:disabled="controlsDisabled"
							@select="selectVersion(activeRow.id, $event)"
						/>
						<IconButton
							ref="changelogCloseButton"
							type="quiet"
							size="sm"
							:label="formatMessage(messages.closeChangelog)"
							@click="closeChangelog"
						>
							<XIcon aria-hidden="true" />
						</IconButton>
					</div>
					<div
						:key="activeRow.version.id"
						class="relative min-h-0 flex-1 overflow-y-auto p-4 pb-14"
						:aria-busy="loadingChangelog"
					>
						<div v-if="loadingChangelog" class="flex items-center justify-center gap-2 p-6 text-secondary" role="status">
							<SpinnerIcon class="size-6 animate-spin" aria-hidden="true" />
							{{ formatMessage(messages.loadingChangelog) }}
						</div>
						<div
							v-else-if="activeRow.version.changelog"
							class="markdown-body [&>:first-child]:!mt-0"
							v-html="renderHighlightedString(activeRow.version.changelog)"
						/>
						<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noChangelog) }}</p>
					</div>
					<div class="pointer-events-none absolute inset-x-0 bottom-0 h-14 bg-gradient-to-t from-surface-2 to-transparent" />
				</Motion>
			</AnimatePresence>
		</div>

		<div class="flex flex-wrap items-center justify-between gap-4 border-0 border-t border-solid border-surface-5 p-6">
			<div class="flex min-w-[240px] flex-1 items-start gap-2">
				<CircleAlertIcon class="size-6 shrink-0 text-orange" aria-hidden="true" />
				<p class="m-0 leading-6">{{ formatMessage(messages.warning) }}</p>
			</div>
			<div class="ml-auto flex shrink-0 items-center gap-2">
				<Button type="outlined" :disabled="actionLoading" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="updateDisabled" :loading="actionLoading" @click="update">
					<DownloadIcon aria-hidden="true" />
					<span class="grid tabular-nums">
						<span
							v-for="count in reservedUpdateLabelCounts"
							:key="count"
							aria-hidden="true"
							class="invisible col-start-1 row-start-1 whitespace-nowrap"
						>
							{{ formatMessage(messages.update, { count }) }}
						</span>
						<span class="col-start-1 row-start-1 justify-self-center">
							{{ formatMessage(messages.update, { count: selections.length }) }}
						</span>
					</span>
				</Button>
			</div>
		</div>
	</NewModal>
</template>
