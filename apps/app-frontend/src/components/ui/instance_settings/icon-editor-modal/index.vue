<script setup lang="ts">
import { CheckIcon, InfoIcon, RefreshCwIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

import { toError } from '@/helpers/errors'
import {
	cache_generated_icon,
	edit_generated_icon,
	edit_generated_icon_if_empty,
	get_recent_icon_configs,
} from '@/helpers/instance'
import type { IconBackground, InstanceIconConfig } from '@/helpers/types'

import {
	type BackgroundId,
	backgroundOptions,
	DEFAULT_BACKGROUND_ID,
	DEFAULT_SYMBOL_ID,
	RANDOM_CONFIG_BLACKLIST,
	type SymbolId,
	type SymbolOption,
	symbolOptions,
} from './editor-catalog'

const props = defineProps<{
	instanceId?: string
	config?: InstanceIconConfig | null
}>()

const emit = defineEmits<{
	saved: [iconPath: string, config: InstanceIconConfig]
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const recentConfigs = ref<InstanceIconConfig[]>([])
const saving = ref(false)
const backgroundScroller = ref<HTMLElement | null>(null)
const showLeftBackgroundShadow = ref(false)
const showRightBackgroundShadow = ref(false)
const draggingBackgrounds = ref(false)

let backgroundScrollerResizeObserver: ResizeObserver | null = null
let backgroundDragPointerId: number | null = null
let backgroundDragCaptureTarget: Element | null = null
let backgroundDragStartX = 0
let backgroundDragStartScrollLeft = 0
let suppressBackgroundClick = false
let suppressBackgroundClickTimeout: ReturnType<typeof setTimeout> | null = null

const selectedBackground = ref<BackgroundId>(DEFAULT_BACKGROUND_ID)
const selectedSymbol = ref<SymbolId>(DEFAULT_SYMBOL_ID)

const selectedBackgroundOption = computed(
	() => backgroundOptions.find((option) => option.id === selectedBackground.value)!,
)
const selectedSymbolOption = computed(
	() => symbolOptions.find((option) => option.id === selectedSymbol.value)!,
)
const vanillaSymbolStartIndex = symbolOptions.findIndex((option) => option.category === 'vanilla')
const selectedConfig = computed<InstanceIconConfig>(() => ({
	background: { ...selectedBackgroundOption.value.background },
	symbol: selectedSymbol.value,
}))
const visibleRecentConfigs = computed(() =>
	recentConfigs.value.filter(
		(config) => backgroundOption(config.background) && symbolOption(config.symbol),
	),
)

function updateBackgroundScrollShadows() {
	const el = backgroundScroller.value
	if (!el) {
		showLeftBackgroundShadow.value = false
		showRightBackgroundShadow.value = false
		return
	}

	showLeftBackgroundShadow.value = el.scrollLeft > 0
	showRightBackgroundShadow.value = el.scrollLeft < el.scrollWidth - el.clientWidth - 1
}

function onBackgroundWheel(event: WheelEvent) {
	const el = backgroundScroller.value
	if (!el || el.scrollWidth <= el.clientWidth) return

	const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY
	el.scrollLeft += delta
}

function onBackgroundPointerDown(event: PointerEvent) {
	const el = backgroundScroller.value
	if (!el || event.pointerType === 'touch' || event.button !== 0) return

	backgroundDragPointerId = event.pointerId
	backgroundDragStartX = event.clientX
	backgroundDragStartScrollLeft = el.scrollLeft
	suppressBackgroundClick = false
	backgroundDragCaptureTarget =
		event.target instanceof Element ? (event.target.closest('button') ?? el) : el
	backgroundDragCaptureTarget.setPointerCapture(event.pointerId)
}

function onBackgroundPointerMove(event: PointerEvent) {
	const el = backgroundScroller.value
	if (!el || event.pointerId !== backgroundDragPointerId) return

	const distance = event.clientX - backgroundDragStartX
	if (!draggingBackgrounds.value && Math.abs(distance) < 4) return

	draggingBackgrounds.value = true
	suppressBackgroundClick = true
	event.preventDefault()
	el.scrollLeft = backgroundDragStartScrollLeft - distance
}

function finishBackgroundDrag(event: PointerEvent) {
	const el = backgroundScroller.value
	if (!el || event.pointerId !== backgroundDragPointerId) return

	if (backgroundDragCaptureTarget?.hasPointerCapture(event.pointerId)) {
		backgroundDragCaptureTarget.releasePointerCapture(event.pointerId)
	}
	backgroundDragPointerId = null
	backgroundDragCaptureTarget = null
	draggingBackgrounds.value = false

	if (suppressBackgroundClick) {
		if (suppressBackgroundClickTimeout) clearTimeout(suppressBackgroundClickTimeout)
		suppressBackgroundClickTimeout = setTimeout(() => {
			suppressBackgroundClick = false
			suppressBackgroundClickTimeout = null
		}, 0)
	}
}

function onBackgroundClick(event: MouseEvent) {
	if (!suppressBackgroundClick) return

	event.preventDefault()
	event.stopPropagation()
	suppressBackgroundClick = false
	if (suppressBackgroundClickTimeout) clearTimeout(suppressBackgroundClickTimeout)
	suppressBackgroundClickTimeout = null
}

onMounted(() => {
	backgroundScrollerResizeObserver = new ResizeObserver(updateBackgroundScrollShadows)
	if (backgroundScroller.value) backgroundScrollerResizeObserver.observe(backgroundScroller.value)
	nextTick(updateBackgroundScrollShadows)
})

onBeforeUnmount(() => {
	backgroundScrollerResizeObserver?.disconnect()
	if (suppressBackgroundClickTimeout) clearTimeout(suppressBackgroundClickTimeout)
})

function backgroundOption(background?: IconBackground) {
	if (background?.type !== 'linear-top-down-gradient') return undefined
	return backgroundOptions.find(
		(option) =>
			option.background.top_color === background.top_color &&
			option.background.bottom_color === background.bottom_color,
	)
}

function backgroundStyle(background: IconBackground) {
	if (background.type === 'color') return { backgroundColor: background.value }
	return {
		backgroundImage: `linear-gradient(to bottom, ${background.top_color}, ${background.bottom_color})`,
	}
}

function backgroundKey(background: IconBackground) {
	if (background.type === 'color') return `${background.type}-${background.value}`
	return `${background.type}-${background.top_color}-${background.bottom_color}`
}

function symbolOption(symbol: string) {
	return symbolOptions.find((option) => option.id === symbol)
}

function selectRecent(config: InstanceIconConfig) {
	const background = backgroundOption(config.background)
	const symbol = symbolOption(config.symbol)
	if (!background || !symbol) return

	selectedBackground.value = background.id
	selectedSymbol.value = symbol.id
}

function surpriseMe() {
	const configurations = backgroundOptions.flatMap((background) =>
		symbolOptions
			.filter(
				(symbol: SymbolOption) =>
					!symbol.excludeFromRandomization &&
					background.id !== selectedBackground.value &&
					symbol.id !== selectedSymbol.value &&
					!RANDOM_CONFIG_BLACKLIST.some(
						(config) => config.background === background.id && config.symbol === symbol.id,
					),
			)
			.map((symbol) => ({ background: background.id, symbol: symbol.id })),
	)
	const configuration = configurations[Math.floor(Math.random() * configurations.length)]

	if (!configuration) return
	selectedBackground.value = configuration.background
	selectedSymbol.value = configuration.symbol
}

async function loadRecents() {
	try {
		recentConfigs.value = await get_recent_icon_configs()
	} catch (error) {
		handleError(toError(error))
	}
}

function show() {
	selectedBackground.value = backgroundOption(props.config?.background)?.id ?? DEFAULT_BACKGROUND_ID
	selectedSymbol.value = symbolOption(props.config?.symbol ?? '')?.id ?? DEFAULT_SYMBOL_ID
	modal.value?.show()
	void loadRecents()
	nextTick(updateBackgroundScrollShadows)
}

function hide() {
	modal.value?.hide()
}

async function loadSymbolBytes(asset: string): Promise<number[]> {
	const response = await fetch(asset)
	if (!response.ok) throw new Error('Failed to load the icon symbol.')

	return Array.from(new Uint8Array(await response.arrayBuffer()))
}

async function saveIcon() {
	if (saving.value) return

	saving.value = true
	try {
		const config = selectedConfig.value
		const symbolBytes = await loadSymbolBytes(selectedSymbolOption.value.asset)
		const iconPath = props.instanceId
			? await edit_generated_icon(props.instanceId, config, symbolBytes)
			: await cache_generated_icon(config, symbolBytes, true)
		emit('saved', iconPath, config)
		saving.value = false
		await nextTick()
		hide()
	} catch (error) {
		handleError(toError(error))
	} finally {
		saving.value = false
	}
}

async function randomizeAndSave() {
	try {
		surpriseMe()
		const config = selectedConfig.value
		const iconPath = await cache_generated_icon(
			config,
			await loadSymbolBytes(selectedSymbolOption.value.asset),
		)
		return { iconPath, config }
	} catch (error) {
		handleError(toError(error))
		return null
	}
}

async function applyGeneratedIcon(instanceId: string, config: InstanceIconConfig) {
	try {
		const symbol = symbolOption(config.symbol)
		if (!backgroundOption(config.background) || !symbol) return false

		await edit_generated_icon_if_empty(instanceId, config, await loadSymbolBytes(symbol.asset))
		return true
	} catch (error) {
		handleError(toError(error))
		return false
	}
}

defineExpose({ show, hide, randomize: randomizeAndSave, randomizeAndSave, applyGeneratedIcon })

const messages = defineMessages({
	title: {
		id: 'instance.icon-editor.title',
		defaultMessage: 'Icon editor',
	},
	background: {
		id: 'instance.icon-editor.background',
		defaultMessage: 'Background',
	},
	symbol: {
		id: 'instance.icon-editor.symbol',
		defaultMessage: 'Symbol',
	},
	surpriseMe: {
		id: 'instance.icon-editor.surprise-me',
		defaultMessage: 'Randomize',
	},
	recents: {
		id: 'instance.icon-editor.recents',
		defaultMessage: 'Recents',
	},
	description: {
		id: 'instance.icon-editor.description',
		defaultMessage: 'Mix and match elements to create a custom icon.',
	},
	saveIcon: {
		id: 'instance.icon-editor.save',
		defaultMessage: 'Save icon',
	},
})
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		width="928px"
		max-width="calc(100vw - 2rem)"
		no-padding
		actions-divider
		:disable-close="saving"
	>
		<div class="flex h-[552px] max-h-[calc(100vh-168px)] min-h-0">
			<aside
				class="flex w-[244px] shrink-0 flex-col gap-4 overflow-y-auto border-0 border-r border-solid border-surface-5 p-6"
			>
				<div
					class="flex w-full flex-col items-center gap-3.5 rounded-[20px] border border-solid border-surface-4 bg-surface-2 p-5"
				>
					<Avatar
						:src="selectedSymbolOption.asset"
						size="132px"
						:style="backgroundStyle(selectedBackgroundOption.background)"
						no-shadow
					/>
					<div class="flex items-center gap-2.5">
						<Avatar
							:src="selectedSymbolOption.asset"
							size="40px"
							:style="backgroundStyle(selectedBackgroundOption.background)"
							no-shadow
						/>
						<Avatar
							:src="selectedSymbolOption.asset"
							size="30px"
							:style="backgroundStyle(selectedBackgroundOption.background)"
							no-shadow
						/>
						<Avatar
							:src="selectedSymbolOption.asset"
							size="20px"
							:style="backgroundStyle(selectedBackgroundOption.background)"
							no-shadow
						/>
					</div>
				</div>

				<Button class="w-full" @click="surpriseMe">
					<RefreshCwIcon />
					{{ formatMessage(messages.surpriseMe) }}
				</Button>

				<div v-if="visibleRecentConfigs.length" class="flex flex-col gap-2.5">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.recents) }}</span>
					<div class="grid grid-cols-4 gap-3">
						<button
							v-for="(recentConfig, index) in visibleRecentConfigs"
							:key="`${backgroundKey(recentConfig.background)}-${recentConfig.symbol}`"
							class="icon-outline relative size-10 cursor-pointer overflow-hidden rounded-xl border-0 p-0 transition-transform hover:scale-105"
							:style="backgroundStyle(recentConfig.background)"
							:aria-label="`${formatMessage(messages.recents)} ${index + 1}`"
							@click="selectRecent(recentConfig)"
						>
							<img
								:src="symbolOption(recentConfig.symbol)?.asset"
								alt=""
								class="size-full object-cover"
							/>
						</button>
					</div>
				</div>
			</aside>

			<div class="min-w-0 flex-1 overflow-y-auto bg-surface-2">
				<section class="border-0 border-b border-solid border-surface-5 p-4">
					<h3 class="m-0 mb-3 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.background) }}
					</h3>
					<div class="relative">
						<div
							class="background-scroll-shadow-left pointer-events-none absolute bottom-0 -left-0.5 top-0 z-10 w-8 bg-surface-2 transition-opacity duration-200"
							:class="showLeftBackgroundShadow ? 'opacity-100' : 'opacity-0'"
						/>
						<div
							ref="backgroundScroller"
							class="flex w-full select-none gap-2.5 overflow-x-auto overflow-y-hidden pb-2 pr-6"
							:class="{ 'cursor-grabbing': draggingBackgrounds }"
							@pointerdown="onBackgroundPointerDown"
							@pointermove="onBackgroundPointerMove"
							@pointerup="finishBackgroundDrag"
							@pointercancel="finishBackgroundDrag"
							@click.capture="onBackgroundClick"
							@wheel.prevent="onBackgroundWheel"
							@scroll="updateBackgroundScrollShadows"
						>
							<button
								v-for="option in backgroundOptions"
								:key="option.id"
								class="icon-option icon-outline relative aspect-square w-[calc((100%_-_3.125rem)/6)] shrink-0 cursor-pointer rounded-[20px] border-0 p-0"
								:class="{ 'icon-outline-selected': selectedBackground === option.id }"
								:style="backgroundStyle(option.background)"
								:aria-label="formatMessage(option.name)"
								:aria-pressed="selectedBackground === option.id"
								@click="selectedBackground = option.id"
							>
								<span
									v-if="selectedBackground === option.id"
									class="absolute right-1.5 top-1.5 flex size-6 items-center justify-center rounded-full bg-white/80 text-black"
								>
									<CheckIcon class="size-4" />
								</span>
							</button>
						</div>
						<div
							class="background-scroll-shadow-right pointer-events-none absolute bottom-0 right-0 top-0 z-10 w-8 bg-surface-2 transition-opacity duration-200"
							:class="showRightBackgroundShadow ? 'opacity-100' : 'opacity-0'"
						/>
					</div>
				</section>

				<section class="p-4">
					<h3 class="m-0 mb-3 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.symbol) }}
					</h3>
					<div class="grid grid-cols-6 gap-2.5">
						<template v-for="(option, index) in symbolOptions" :key="option.id">
							<hr
								v-if="index === vanillaSymbolStartIndex"
								class="col-span-6 my-2.5 mx-1 w-full border-0 border-t border-solid border-surface-5"
							/>
							<button
								v-tooltip="{
									content: formatMessage(option.name),
									delay: { show: 500, hide: 0 },
								}"
								class="icon-option icon-outline relative aspect-square cursor-pointer overflow-hidden rounded-[20px] border-0 bg-transparent p-0"
								:class="{ 'icon-outline-selected': selectedSymbol === option.id }"
								:aria-label="formatMessage(option.name)"
								:aria-pressed="selectedSymbol === option.id"
								@click="selectedSymbol = option.id"
							>
								<img :src="option.asset" alt="" class="size-full object-cover" />
								<span
									v-if="selectedSymbol === option.id"
									class="absolute right-1.5 top-1.5 flex size-6 items-center justify-center rounded-full bg-white/80 text-black"
								>
									<CheckIcon class="size-4" />
								</span>
							</button>
						</template>
					</div>
				</section>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-between gap-4 px-2">
				<div class="flex min-w-0 items-center gap-2 text-primary">
					<InfoIcon class="size-6 shrink-0 text-blue" />
					<span>{{ formatMessage(messages.description) }}</span>
				</div>
				<div class="flex shrink-0 items-center gap-2">
					<Button :disabled="saving" type="outlined" @click="hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button type="colored" color="brand" :disabled="saving" @click="saveIcon">
						<SpinnerIcon v-if="saving" class="animate-spin" />
						<SaveIcon v-else />
						{{ formatMessage(messages.saveIcon) }}
					</Button>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<style scoped>
.icon-outline {
	outline: 1px solid color-mix(in srgb, var(--color-text-primary) 15%, transparent);
	outline-offset: -1px;
}

.icon-option:not(.icon-outline-selected):hover {
	outline-color: color-mix(in srgb, var(--color-text-primary) 30%, transparent);
}

.icon-option {
	transition: outline-color 150ms ease;
}

.cursor-grabbing,
.cursor-grabbing * {
	cursor: grabbing !important;
}

.icon-outline-selected {
	outline-color: color-mix(in srgb, var(--color-text-primary) 60%, transparent);
}

.background-scroll-shadow-left {
	-webkit-mask-image: linear-gradient(to right, black, transparent);
	mask-image: linear-gradient(to right, black, transparent);
}

.background-scroll-shadow-right {
	-webkit-mask-image: linear-gradient(to left, black, transparent);
	mask-image: linear-gradient(to left, black, transparent);
}
</style>
