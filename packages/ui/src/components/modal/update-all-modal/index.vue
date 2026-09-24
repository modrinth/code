<script setup lang="ts">
import {
	CircleAlertIcon,
	DownloadIcon,
	FileTextIcon,
	PanelRightCloseIcon,
	RightArrowIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { renderHighlightedString } from '@modrinth/utils'
import { useElementSize, useMediaQuery } from '@vueuse/core'
import { AnimatePresence, Motion, useReducedMotion } from 'motion-v'
import { computed, nextTick, ref, useId, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { useVIntl } from '#ui/composables/i18n'
import { useScrollIndicator } from '#ui/composables/scroll-indicator'
import { dismissTooltip } from '#ui/providers/tooltip'
import { commonMessages } from '#ui/utils/common-messages'
import { getModifiedSelection } from '#ui/utils/modified-selection'

import { messages } from './update-all-modal-messages'
import UpdateAllModalTruncatedProjectTitle from './update-all-modal-truncated-project-title.vue'
import UpdateAllModalTruncatedVersion from './update-all-modal-truncated-version.vue'
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
	preloadChangelog: [selection: UpdateAllSelection]
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const contentContainer = ref<HTMLElement | null>(null)
const tableContainer = ref<HTMLElement | null>(null)
const modalHeading = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade } = useScrollIndicator(tableContainer)
const changelogItemId = ref<string>()
const changelogEntered = ref(false)
const changelogHeadingId = useId()
const isOpen = ref(false)
const selectionAnchorId = ref<string>()
const focusAfterChangelogExit = ref<string>()
const prefersReducedMotion = useReducedMotion()
const { width: contentWidth } = useElementSize(contentContainer)
const isNarrowViewport = useMediaQuery('(max-width: 751px)')
let submitted = false

const {
	rows,
	selections,
	allSelected,
	indeterminate,
	selectItem,
	selectAll,
	selectVersion,
	reset,
} = useUpdateAllSelection(() => props.items)

const activeRow = computed(() =>
	props.loading
		? undefined
		: rows.value.find((row) => row.id === changelogItemId.value && row.version),
)
const controlsDisabled = computed(() => props.loading || props.actionLoading)
const availableCount = computed(() => rows.value.filter((row) => row.version).length)
const listStatus = computed(() => {
	if (props.loading) return formatMessage(messages.loading)
	if (!rows.value.length) return formatMessage(messages.empty)
	if (!availableCount.value) return formatMessage(messages.noCompatibleUpdates)
	return formatMessage(messages.results, { count: availableCount.value })
})
const updateDisabled = computed(
	() => controlsDisabled.value || props.actionDisabled || selections.value.length === 0,
)
const reservedUpdateLabelCounts = computed(() => [...new Set([0, 1, 2, rows.value.length])])
const noChangelogMotion = computed(
	() =>
		(contentWidth.value ? contentWidth.value < 720 : isNarrowViewport.value) ||
		!!prefersReducedMotion.value,
)

const compactTable = computed(
	() => !!activeRow.value && (noChangelogMotion.value || changelogEntered.value),
)

watch(
	() => !!activeRow.value,
	(open) => {
		if (!open) {
			changelogEntered.value = false
			if (isOpen.value && changelogItemId.value) {
				focusAfterChangelogExit.value = changelogItemId.value
				changelogItemId.value = undefined
			}
		}
	},
	{ flush: 'sync' },
)

function handleChangelogAnimationComplete(definition: unknown) {
	if (definition === 'open' && activeRow.value) changelogEntered.value = true
}

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
	if (activeRow.value?.id === id) {
		if (tableContainer.value) tableContainer.value.scrollLeft = 0
		document.getElementById(changelogHeadingId)?.focus({ preventScroll: true })
	}
}

function preloadChangelog(id: string) {
	if (!isOpen.value || controlsDisabled.value) return
	const row = rows.value.find((row) => row.id === id)
	if (!row?.version || row.version.changelog != null) return
	emit('preloadChangelog', { id: row.id, projectId: row.project.id, version: row.version })
}

function closeChangelog() {
	focusAfterChangelogExit.value = changelogItemId.value
	changelogItemId.value = undefined
}

function handleChangelogExitComplete() {
	const id = focusAfterChangelogExit.value
	if (!id || activeRow.value || !isOpen.value) return
	focusAfterChangelogExit.value = undefined
	const trigger = Array.from(tableContainer.value?.querySelectorAll<HTMLElement>('[data-changelog-id]') ?? [])
		.find(
			(button) =>
				button.dataset.changelogId === id &&
				!button.hasAttribute('disabled') &&
				button.getClientRects().length > 0,
		)
	const focusTarget = trigger ?? modalHeading.value
	focusTarget?.focus({ preventScroll: true })
}

function initialFocus() {
	return activeRow.value ? document.getElementById(changelogHeadingId) : modalHeading.value
}

function show(options?: { changelogItemId?: string }) {
	if (isOpen.value) return
	reset()
	changelogEntered.value = false
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
	const next = getModifiedSelection(selectableIds, selectedIds, id, selectionAnchorId.value, event)
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

function handleRowClick(id: string, event: MouseEvent) {
	if (event.button !== 0 || isInteractiveRowTarget(event)) return
	if (!controlsDisabled.value && rows.value.some((row) => row.id === id && row.version)) {
		openChangelog(id)
	}
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
		width="1200px"
		max-width="1200px"
		class="update-all-modal @container !rounded-[20px] !bg-surface-3"
		no-padding
		:disable-close="actionLoading"
		:initial-focus="initialFocus"
		:on-hide="handleHide"
	>
		<template #title>
			<h2
				ref="modalHeading"
				tabindex="-1"
				class="m-0 text-2xl font-semibold text-contrast focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
			>
				{{ formatMessage(messages.header) }}
			</h2>
		</template>
		<div role="status" class="sr-only">
			{{ listStatus }}
		</div>
		<div
			ref="contentContainer"
			class="relative flex h-[643px] max-h-[643px] min-h-0 shrink overflow-clip bg-surface-2"
		>
			<div
				class="flex w-full min-h-0 min-w-0 flex-col bg-surface-3"
				:class="{
					'hidden @[720px]:flex @[720px]:w-[240px] @[940px]:w-[304px] @[1080px]:w-[360px]':
						compactTable,
				}"
				:aria-busy="loading"
			>
				<div
					v-if="!loading && rows.length"
					class="update-all-modal-scroll-header relative z-[1] grid shrink-0 border-0 border-b border-solid border-surface-5 bg-surface-3"
					:class="
						compactTable
							? 'grid-cols-1'
							: '@[720px]:grid-cols-[240px_minmax(0,1fr)_120px] @[900px]:grid-cols-[240px_minmax(0,1fr)_128px] @[940px]:grid-cols-[304px_minmax(0,1fr)_128px] @[1080px]:grid-cols-[360px_minmax(0,1fr)_128px]'
					"
					:data-show-fade="showTopFade"
				>
					<div class="flex items-center gap-3 px-4 py-3 font-semibold">
						<Checkbox
							class="[&>span:first-child]:!size-6 [&>span:first-child]:!rounded-lg"
							:model-value="allSelected"
							:indeterminate="indeterminate"
							:description="formatMessage(messages.selectAll)"
							:disabled="controlsDisabled || !rows.some((row) => row.version)"
							@update:model-value="handleSelectAll"
						/>
						<span class="hidden @[720px]:inline">{{ formatMessage(messages.project) }}</span>
						<span class="@[720px]:hidden">{{ formatMessage(messages.selectAll) }}</span>
					</div>
					<div v-if="!compactTable" class="hidden items-center px-4 py-3 font-semibold @[720px]:flex">
						{{ formatMessage(messages.versions) }}
					</div>
					<div
						v-if="!compactTable"
						class="hidden items-center justify-end truncate px-4 py-3 text-right font-semibold @[720px]:flex"
					>
						{{ formatMessage(commonMessages.changelogLabel) }}
					</div>
				</div>
				<div
					ref="tableContainer"
					class="min-h-0 flex-1 overflow-auto bg-surface-3 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-inset focus-visible:ring-brand-shadow"
					role="region"
					:aria-label="formatMessage(messages.projectList)"
					:tabindex="!loading && rows.length ? 0 : -1"
				>
					<template v-if="!loading && rows.length">
						<table
							class="hidden w-full table-fixed border-collapse text-left @[720px]:table"
							:aria-label="formatMessage(messages.header)"
						>
							<colgroup>
								<col
									:class="
										compactTable ? 'w-full' : 'w-[240px] @[940px]:w-[304px] @[1080px]:w-[360px]'
									"
								/>
								<col v-if="!compactTable" />
								<col v-if="!compactTable" class="w-[120px] @[900px]:w-[128px]" />
							</colgroup>
							<thead class="sr-only">
								<tr>
									<th scope="col">{{ formatMessage(messages.project) }}</th>
									<th v-if="!compactTable" scope="col">{{ formatMessage(messages.versions) }}</th>
									<th v-if="!compactTable" scope="col">
										{{ formatMessage(commonMessages.changelogLabel) }}
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
										row.version && !controlsDisabled
											? 'cursor-pointer transition-colors hover:bg-surface-3'
											: '',
									]"
									@mouseenter="preloadChangelog(row.id)"
									@click="handleRowClick(row.id, $event)"
								>
									<th scope="row" :aria-label="row.project.title" class="px-4 py-3 font-normal">
										<div class="flex min-w-0 items-center gap-3">
											<Checkbox
												class="shrink-0 [&>span:first-child]:!size-6 [&>span:first-child]:!rounded-lg"
												:model-value="row.selected"
												:description="
													formatMessage(messages.selectProject, { project: row.project.title })
												"
												:disabled="controlsDisabled || !row.version"
												@update:model-value="
													(selected, event) => handleCheckboxSelection(row.id, selected, event)
												"
											/>
											<div class="flex min-w-0 items-center gap-2">
												<Avatar :src="row.project.icon_url" size="28px" class="!rounded-lg" />
												<UpdateAllModalTruncatedProjectTitle
													as="button"
													:title="row.project.title"
													type="button"
													class="min-w-0 cursor-pointer truncate border-0 bg-transparent p-0 text-left font-medium text-contrast hover:underline focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
													:disabled="!row.version || controlsDisabled"
													:aria-label="
														formatMessage(messages.viewChangelog, { project: row.project.title })
													"
													:data-changelog-id="row.id"
													@focus="preloadChangelog(row.id)"
													@click="openChangelog(row.id)"
												/>
											</div>
										</div>
									</th>
									<td
										v-show="!compactTable"
										:inert="!!activeRow || !!focusAfterChangelogExit"
										class="px-4 py-3"
									>
										<div class="flex min-w-0 items-center gap-2">
											<div class="w-[120px] min-w-0 shrink-0 @[800px]:w-[150px] @[900px]:w-[190px]">
												<span class="sr-only">{{ formatMessage(messages.currentVersion) }}: </span>
												<UpdateAllModalTruncatedVersion
													:version="row.currentVersion.version_number"
												/>
											</div>
											<RightArrowIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
											<div class="w-[152px] min-w-0 shrink-0 @[800px]:w-[180px] @[900px]:w-[220px]">
												<span v-if="row.versions.length === 1" class="sr-only">
													{{ formatMessage(messages.newVersion) }}:
												</span>
												<UpdateAllModalVersionSelect
													v-if="row.version && !compactTable"
													:versions="row.versions"
													:version="row.version"
													:label="
														formatMessage(messages.selectVersion, { project: row.project.title })
													"
													:disabled="controlsDisabled"
													@select="selectVersion(row.id, $event)"
												/>
												<span
													v-else-if="!row.version"
													class="block truncate text-sm text-secondary"
													>{{ formatMessage(messages.noVersion) }}</span
												>
											</div>
										</div>
									</td>
									<td
										v-show="!compactTable"
										:inert="!!activeRow || !!focusAfterChangelogExit"
										class="px-4 py-3"
									>
										<div class="flex justify-end">
											<IconButton
												v-tooltip="
													formatMessage(messages.viewChangelog, { project: row.project.title })
												"
												size="sm"
												:label="
													formatMessage(messages.viewChangelog, { project: row.project.title })
												"
												:disabled="!row.version || controlsDisabled"
												@focus="preloadChangelog(row.id)"
												@click="openChangelog(row.id)"
											>
												<FileTextIcon aria-hidden="true" />
											</IconButton>
										</div>
									</td>
								</tr>
							</tbody>
						</table>
						<div class="@[720px]:hidden">
							<div class="flex flex-col gap-3 p-3">
								<div
									v-for="row in rows"
									:key="row.id"
									class="min-w-0 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-4"
									role="group"
									:aria-label="row.project.title"
									:class="
										row.version && !controlsDisabled
											? 'cursor-pointer transition-colors hover:bg-surface-3'
											: ''
									"
									@mouseenter="preloadChangelog(row.id)"
									@click="handleRowClick(row.id, $event)"
								>
									<div class="flex min-w-0 items-center gap-3">
										<Checkbox
											class="shrink-0 [&>span:first-child]:!size-6 [&>span:first-child]:!rounded-lg"
											:model-value="row.selected"
											:description="
												formatMessage(messages.selectProject, { project: row.project.title })
											"
											:disabled="controlsDisabled || !row.version"
											@update:model-value="
												(selected, event) => handleCheckboxSelection(row.id, selected, event)
											"
										/>
										<Avatar :src="row.project.icon_url" size="28px" class="!rounded-lg" />
										<button
											type="button"
											class="min-w-0 flex-1 cursor-pointer break-words border-0 bg-transparent p-0 text-left font-medium text-contrast hover:underline focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
											:disabled="!row.version || controlsDisabled"
											:data-changelog-id="row.id"
											:aria-label="
												formatMessage(messages.viewChangelog, { project: row.project.title })
											"
											@focus="preloadChangelog(row.id)"
											@click="openChangelog(row.id)"
										>
											{{ row.project.title }}
										</button>
										<IconButton
											v-tooltip="
												formatMessage(messages.viewChangelog, { project: row.project.title })
											"
											size="md"
											:label="formatMessage(messages.viewChangelog, { project: row.project.title })"
											:disabled="!row.version || controlsDisabled"
											@focus="preloadChangelog(row.id)"
											@click="openChangelog(row.id)"
										>
											<FileTextIcon aria-hidden="true" />
										</IconButton>
									</div>
									<div class="mt-4 flex min-w-0 flex-col gap-3">
										<div class="min-w-0">
											<div class="text-sm font-semibold text-secondary">
												{{ formatMessage(messages.currentVersion) }}
											</div>
											<div class="mt-1 break-all text-sm">
												{{ row.currentVersion.version_number }}
											</div>
										</div>
										<div class="min-w-0">
											<div class="text-sm font-semibold text-secondary">
												{{ formatMessage(messages.newVersion) }}
											</div>
											<UpdateAllModalVersionSelect
												v-if="row.version && !compactTable"
												class="mt-1 w-full"
												:versions="row.versions"
												:version="row.version"
												:label="
													formatMessage(messages.selectVersion, { project: row.project.title })
												"
												:disabled="controlsDisabled"
												wrap
												@select="selectVersion(row.id, $event)"
											/>
											<div v-else-if="!row.version" class="mt-1 text-sm text-secondary">
												{{ formatMessage(messages.noVersion) }}
											</div>
										</div>
									</div>
								</div>
							</div>
						</div>
					</template>
					<div
						v-else
						class="flex h-full items-center justify-center gap-2 p-6 text-secondary"
					>
						<SpinnerIcon v-if="loading" class="size-6 animate-spin" aria-hidden="true" />
						{{ formatMessage(loading ? messages.loading : messages.empty) }}
					</div>
				</div>
			</div>
			<div
				aria-hidden="true"
				class="pointer-events-none absolute inset-x-0 bottom-0 h-6 bg-gradient-to-t from-surface-2 to-transparent transition-opacity duration-150"
				:class="showBottomFade ? 'opacity-100' : 'opacity-0'"
			/>

			<AnimatePresence :initial="false" :on-exit-complete="handleChangelogExitComplete">
				<Motion
					v-if="activeRow && activeRow.version"
					key="changelog-panel"
					as="section"
					class="@container absolute inset-y-0 right-0 z-10 flex w-full min-w-0 flex-col border-0 border-l border-solid border-surface-5 bg-surface-2 @[720px]:w-[calc(100%_-_240px)] @[940px]:w-[calc(100%_-_304px)] @[1080px]:w-[calc(100%_-_360px)]"
					:aria-labelledby="changelogHeadingId"
					:initial="noChangelogMotion ? { x: 0, opacity: 1 } : { x: '100%', opacity: 0 }"
					animate="open"
					:variants="{ open: { x: 0, opacity: 1 } }"
					:on-animation-complete="handleChangelogAnimationComplete"
					:exit="noChangelogMotion ? { x: 0, opacity: 1 } : { x: '100%', opacity: 0 }"
					:transition="{ duration: noChangelogMotion ? 0 : 0.3, ease: 'easeInOut' }"
				>
					<div role="status" class="sr-only">
						{{
							formatMessage(
								loadingChangelog
									? messages.loadingChangelog
									: activeRow.version.changelog
										? messages.changelogLoaded
										: messages.noChangelog,
							)
						}}
					</div>
					<div
						class="grid min-h-[68px] grid-cols-[minmax(0,1fr)_auto] items-center gap-3 border-0 border-b border-solid border-surface-5 p-4 @[560px]:flex @[560px]:flex-wrap"
					>
						<div class="order-1 flex min-w-0 items-center gap-2 @[560px]:flex-1">
							<Avatar :src="activeRow.project.icon_url" size="36px" class="!rounded-lg" />
							<UpdateAllModalTruncatedProjectTitle
								:id="changelogHeadingId"
								as="h3"
								:title="activeRow.project.title"
								:aria-label="
									formatMessage(messages.changelogFor, { project: activeRow.project.title })
								"
								tabindex="-1"
								class="m-0 min-w-0 flex-1 truncate text-xl font-semibold text-contrast focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
							/>
						</div>
						<div
							class="order-3 col-span-2 flex min-w-0 items-center gap-2 @[560px]:order-2 @[560px]:col-span-1"
						>
							<div
								class="min-w-0 flex-1 @[560px]:flex-none"
								:class="activeRow.versions.length > 1 ? '@[560px]:w-[220px]' : ''"
							>
								<UpdateAllModalVersionSelect
									:key="activeRow.id"
									:versions="activeRow.versions"
									:version="activeRow.version"
									:label="
										formatMessage(messages.selectVersion, { project: activeRow.project.title })
									"
									:disabled="controlsDisabled"
									@select="selectVersion(activeRow.id, $event)"
								/>
							</div>
						</div>
						<IconButton
							v-tooltip="formatMessage(messages.closeChangelog)"
							type="quiet"
							size="sm"
							class="order-2 @[560px]:order-3"
							:label="formatMessage(messages.closeChangelog)"
							@click="closeChangelog"
						>
							<PanelRightCloseIcon aria-hidden="true" />
						</IconButton>
					</div>
					<div
						:key="activeRow.version.id"
						class="relative min-h-0 flex-1 overflow-y-auto p-4 pb-14 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-inset focus-visible:ring-brand-shadow"
						role="region"
						:aria-labelledby="changelogHeadingId"
						tabindex="0"
						:aria-busy="loadingChangelog"
					>
						<div
							v-if="loadingChangelog"
							class="flex items-center justify-center gap-2 p-6 text-secondary"
						>
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
					<div
						class="pointer-events-none absolute inset-x-0 bottom-0 h-14 bg-gradient-to-t from-surface-2 to-transparent"
					/>
				</Motion>
			</AnimatePresence>
		</div>

		<div
			class="flex shrink-0 flex-col items-stretch justify-between gap-4 border-0 border-t border-solid border-surface-5 p-4 @[720px]:flex-row @[720px]:items-center @[720px]:p-6"
		>
			<div class="flex min-w-0 flex-1 items-start gap-2">
				<CircleAlertIcon class="size-6 shrink-0 text-orange" aria-hidden="true" />
				<p class="m-0 leading-6">{{ formatMessage(messages.warning) }}</p>
			</div>
			<div
				class="flex w-full min-w-0 flex-col gap-2 @[480px]:w-auto @[480px]:flex-row @[480px]:flex-wrap @[480px]:self-end @[720px]:ml-auto @[720px]:shrink-0"
			>
				<Button
					type="outlined"
					class="w-full justify-center @[480px]:w-auto"
					:disabled="actionLoading"
					@click="hide"
				>
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					class="w-full justify-center @[480px]:w-auto"
					:disabled="updateDisabled"
					:loading="actionLoading"
					@click="update"
				>
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

<style>
.update-all-modal-scroll-header::after {
	position: absolute;
	top: 100%;
	right: 0;
	left: 0;
	height: 1.5rem;
	pointer-events: none;
	content: '';
	background: linear-gradient(to bottom, var(--surface-2), transparent);
	opacity: 0;
}

.update-all-modal-scroll-header[data-show-fade='true']::after {
	opacity: 1;
	transition: opacity 150ms;
}

.update-all-modal > [data-modal-content] {
	display: flex;
	min-height: 0;
	flex-direction: column;
	overflow-y: auto;
}
</style>
