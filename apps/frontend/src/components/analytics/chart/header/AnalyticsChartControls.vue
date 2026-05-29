<template>
	<Menu
		theme="analytics-controls-menu"
		placement="bottom-end"
		:shown="isControlsMenuOpen"
		:triggers="[]"
		:popper-triggers="[]"
		:aria-id="controlsMenuId"
		no-auto-focus
		@update:shown="isControlsMenuOpen = $event"
	>
		<button
			ref="controlsMenuTrigger"
			type="button"
			:aria-expanded="isControlsMenuOpen"
			:aria-controls="controlsMenuId"
			:aria-label="
				formatMessage(analyticsChartMessages.controlsAria, {
					activeCount: activeControlCountLabel,
				})
			"
			class="btn-dropdown-animation inline-flex min-h-5 cursor-pointer items-center justify-between gap-2 rounded-xl border-0 bg-surface-4 px-3 py-2 text-left text-sm font-semibold text-button-text shadow-none transition-all duration-200 hover:brightness-[115%] focus-visible:brightness-[115%] active:brightness-[115%]"
			@click="toggleControlsMenu"
		>
			<Settings2Icon class="size-4 text-secondary" aria-hidden="true" />
			<span class="leading-tight text-primary">
				{{ formatMessage(analyticsChartMessages.controlsButton) }}
			</span>
			<span
				v-if="activeControlCount > 0"
				class="inline-flex min-w-5 items-center justify-center rounded-full bg-highlight-green px-1.5 text-xs font-semibold leading-5 text-green"
			>
				{{ activeControlCount }}
			</span>
			<DropdownIcon class="size-4 text-secondary" aria-hidden="true" />
		</button>
		<template #popper>
			<div
				ref="controlsMenuPanel"
				role="dialog"
				:aria-label="formatMessage(analyticsChartMessages.controlsDialogAria)"
				class="mt-1 flex w-[228px] max-w-[calc(100vw_-_2rem)] flex-col overflow-hidden rounded-[14px] border border-solid border-surface-4 bg-surface-3 text-sm shadow-2xl"
			>
				<div class="flex items-center justify-between gap-3 px-3 py-2.5 text-xs font-medium">
					<span class="font-semibold text-primary">{{ activeControlCountLabel }}</span>
					<button
						type="button"
						:disabled="isResetDisabled"
						class="border-0 bg-transparent p-0 text-xs font-semibold text-primary transition-all disabled:cursor-not-allowed disabled:opacity-50"
						:class="isResetDisabled ? '' : 'hover:text-contrast focus-visible:text-contrast'"
						@click="resetControls"
					>
						{{ formatMessage(analyticsMessages.resetButton) }}
					</button>
				</div>

				<div
					v-if="hasDisplayControls"
					class="flex flex-col gap-1 border-0 border-t border-solid border-surface-4 px-3 py-2.5"
				>
					<div class="mb-0.5 text-xs font-semibold text-secondary">
						{{ formatMessage(analyticsChartMessages.displayControls) }}
					</div>
					<div v-if="canShowPreviousPeriod" class="flex min-h-7 items-center justify-between">
						<label
							:for="previousPeriodToggleId"
							class="flex min-h-7 min-w-0 grow cursor-pointer items-center gap-1.5 pr-3 font-semibold leading-tight text-primary"
						>
							<HistoryIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
							<span class="min-w-0 truncate">
								{{ formatMessage(analyticsChartMessages.previousPeriod) }}
							</span>
						</label>
						<Toggle
							:id="previousPeriodToggleId"
							v-model="showPreviousPeriodModel"
							:small="smallToggles"
						/>
					</div>
					<div v-if="canUseRatioMode" class="flex min-h-7 items-center justify-between">
						<label
							:for="ratioModeToggleId"
							class="flex min-h-7 min-w-0 grow cursor-pointer items-center gap-1.5 pr-3 font-semibold leading-tight text-primary"
						>
							<span
								class="inline-flex size-4 shrink-0 items-center justify-center text-sm font-semibold leading-none text-secondary"
								aria-hidden="true"
							>
								%
							</span>
							<span class="min-w-0 truncate">
								{{ formatMessage(analyticsChartMessages.ratio) }}
							</span>
						</label>
						<Toggle :id="ratioModeToggleId" v-model="ratioModeModel" :small="smallToggles" />
					</div>
				</div>

				<div
					class="flex flex-col gap-1 border-0 border-t border-solid border-surface-4 px-3 py-2.5"
				>
					<div class="mb-0.5 text-xs font-semibold text-secondary">
						{{ formatMessage(analyticsChartMessages.annotations) }}
					</div>
					<div
						v-tooltip="projectEventsDisabledTooltip"
						class="justify3 flex min-h-7 items-center"
						:aria-disabled="!hasProjectEvents"
					>
						<label
							:for="projectEventsToggleId"
							class="flex min-h-7 min-w-0 grow items-center gap-1.5 pr-3 font-semibold leading-tight text-primary"
							:class="hasProjectEvents ? 'cursor-pointer' : 'cursor-not-allowed opacity-60'"
						>
							<TagCategoryFlagIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
							<span class="min-w-0 truncate">
								{{ formatMessage(analyticsChartMessages.projectEvents) }}
							</span>
						</label>
						<Toggle
							:id="projectEventsToggleId"
							v-model="showProjectEventsControlModel"
							:small="smallToggles"
							:disabled="!hasProjectEvents"
						/>
					</div>
					<div
						v-tooltip="modrinthEventsDisabledTooltip"
						class="justify3 flex min-h-7 items-center"
						:aria-disabled="!hasChartEvents"
					>
						<label
							:for="modrinthEventsToggleId"
							class="flex min-h-7 min-w-0 grow items-center gap-1.5 pr-3 font-semibold leading-tight text-primary"
							:class="hasChartEvents ? 'cursor-pointer' : 'cursor-not-allowed opacity-60'"
						>
							<InfoIcon class="size-4 shrink-0 text-blue" aria-hidden="true" />
							<span class="min-w-0 truncate">
								{{ formatMessage(analyticsChartMessages.modrinthEvents) }}
							</span>
						</label>
						<Toggle
							:id="modrinthEventsToggleId"
							v-model="showChartEventsControlModel"
							:small="smallToggles"
							:disabled="!hasChartEvents"
						/>
					</div>
				</div>
			</div>
		</template>
	</Menu>
</template>

<script setup lang="ts">
import {
	DropdownIcon,
	HistoryIcon,
	InfoIcon,
	Settings2Icon,
	TagCategoryFlagIcon,
} from '@modrinth/assets'
import { Toggle, useVIntl } from '@modrinth/ui'
import { Menu } from 'floating-vue'

import { analyticsChartMessages, analyticsMessages } from '../../analytics-messages'

const props = defineProps<{
	ratioMode: boolean
	showChartEvents: boolean
	showProjectEvents: boolean
	showPreviousPeriod: boolean
	canUseRatioMode: boolean
	canShowPreviousPeriod: boolean
	hasChartEvents: boolean
	hasProjectEvents: boolean
	smallToggles: boolean
	defaultRatioMode: boolean
	defaultShowChartEvents: boolean
	defaultShowProjectEvents: boolean
	defaultShowPreviousPeriod: boolean
}>()

const emit = defineEmits<{
	(
		e:
			| 'update:ratioMode'
			| 'update:showChartEvents'
			| 'update:showProjectEvents'
			| 'update:showPreviousPeriod',
		value: boolean,
	): void
}>()

const isControlsMenuOpen = ref(false)
const controlsMenuTrigger = ref<HTMLElement | null>(null)
const controlsMenuPanel = ref<HTMLElement | null>(null)
const controlsMenuId = useId()
const ratioModeToggleId = useId()
const previousPeriodToggleId = useId()
const modrinthEventsToggleId = useId()
const projectEventsToggleId = useId()
const { formatMessage } = useVIntl()

const ratioModeModel = computed({
	get: () => props.ratioMode,
	set: (value: boolean) => emit('update:ratioMode', value),
})
const showChartEventsModel = computed({
	get: () => props.showChartEvents,
	set: (value: boolean) => emit('update:showChartEvents', value),
})
const showChartEventsControlModel = computed({
	get: () => props.hasChartEvents && props.showChartEvents,
	set: (value: boolean) => emit('update:showChartEvents', value),
})
const showProjectEventsModel = computed({
	get: () => props.showProjectEvents,
	set: (value: boolean) => emit('update:showProjectEvents', value),
})
const showProjectEventsControlModel = computed({
	get: () => props.hasProjectEvents && props.showProjectEvents,
	set: (value: boolean) => emit('update:showProjectEvents', value),
})
const showPreviousPeriodModel = computed({
	get: () => props.showPreviousPeriod,
	set: (value: boolean) => emit('update:showPreviousPeriod', value),
})

const hasDisplayControls = computed(() => props.canShowPreviousPeriod || props.canUseRatioMode)
const projectEventsDisabledTooltip = computed(() =>
	props.hasProjectEvents ? undefined : formatMessage(analyticsChartMessages.noProjectEvents),
)
const modrinthEventsDisabledTooltip = computed(() =>
	props.hasChartEvents ? undefined : formatMessage(analyticsChartMessages.noModrinthEvents),
)
const activeControlCount = computed(() => {
	let count = 0
	if (props.canShowPreviousPeriod && props.showPreviousPeriod) count += 1
	if (props.canUseRatioMode && props.ratioMode) count += 1
	if (props.hasProjectEvents && props.showProjectEvents) count += 1
	if (props.hasChartEvents && props.showChartEvents) count += 1
	return count
})
const activeControlCountLabel = computed(() =>
	formatMessage(analyticsChartMessages.activeControlCount, { count: activeControlCount.value }),
)
const isResetDisabled = computed(
	() =>
		props.showPreviousPeriod === props.defaultShowPreviousPeriod &&
		props.ratioMode === props.defaultRatioMode &&
		props.showProjectEvents === props.defaultShowProjectEvents &&
		props.showChartEvents === props.defaultShowChartEvents,
)

function toggleControlsMenu() {
	isControlsMenuOpen.value = !isControlsMenuOpen.value
}

function resetControls() {
	if (isResetDisabled.value) return

	showPreviousPeriodModel.value = props.defaultShowPreviousPeriod
	ratioModeModel.value = props.defaultRatioMode
	showProjectEventsModel.value = props.defaultShowProjectEvents
	showChartEventsModel.value = props.defaultShowChartEvents
}

function onDocumentPointerDown(event: PointerEvent) {
	if (!isControlsMenuOpen.value || !(event.target instanceof Node)) return
	if (controlsMenuTrigger.value?.contains(event.target)) return
	if (controlsMenuPanel.value?.contains(event.target)) return
	isControlsMenuOpen.value = false
}

onMounted(() => {
	document.addEventListener('pointerdown', onDocumentPointerDown, true)
})

onBeforeUnmount(() => {
	document.removeEventListener('pointerdown', onDocumentPointerDown, true)
})
</script>

<style>
.v-popper--theme-analytics-controls-menu .v-popper__inner {
	overflow: visible !important;
	background: transparent !important;
	padding: 0 !important;
	border: 0 !important;
	box-shadow: none !important;
}

.v-popper--theme-analytics-controls-menu .v-popper__arrow-container {
	display: none;
}
</style>
