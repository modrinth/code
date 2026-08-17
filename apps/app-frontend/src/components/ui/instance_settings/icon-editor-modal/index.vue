<script setup lang="ts">
import { CheckIcon, InfoIcon, RefreshCwIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
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
	get_recent_icon_recipes,
} from '@/helpers/instance'
import type { IconBackground, InstanceIconRecipe } from '@/helpers/types'

import {
	type BackgroundId,
	backgroundOptions,
	DEFAULT_BACKGROUND_ID,
	DEFAULT_SYMBOL_ID,
	type SymbolId,
	symbolOptions,
} from './editor-catalog'

const props = defineProps<{
	instanceId?: string
	recipe?: InstanceIconRecipe | null
}>()

const emit = defineEmits<{
	saved: [iconPath: string, recipe: InstanceIconRecipe]
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const recentRecipes = ref<InstanceIconRecipe[]>([])
const saving = ref(false)
const backgroundScroller = ref<HTMLElement | null>(null)
const showLeftBackgroundShadow = ref(false)
const showRightBackgroundShadow = ref(false)

let backgroundScrollerResizeObserver: ResizeObserver | null = null

const selectedBackground = ref<BackgroundId>(DEFAULT_BACKGROUND_ID)
const selectedSymbol = ref<SymbolId>(DEFAULT_SYMBOL_ID)

const selectedBackgroundOption = computed(
	() => backgroundOptions.find((option) => option.id === selectedBackground.value)!,
)
const selectedSymbolOption = computed(
	() => symbolOptions.find((option) => option.id === selectedSymbol.value)!,
)
const selectedRecipe = computed<InstanceIconRecipe>(() => ({
	background: { ...selectedBackgroundOption.value.background },
	symbol: selectedSymbol.value,
}))
const visibleRecentRecipes = computed(() =>
	recentRecipes.value.filter(
		(recipe) => backgroundOption(recipe.background) && symbolOption(recipe.symbol),
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

onMounted(() => {
	backgroundScrollerResizeObserver = new ResizeObserver(updateBackgroundScrollShadows)
	if (backgroundScroller.value) backgroundScrollerResizeObserver.observe(backgroundScroller.value)
	nextTick(updateBackgroundScrollShadows)
})

onBeforeUnmount(() => backgroundScrollerResizeObserver?.disconnect())

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

function selectRecent(recipe: InstanceIconRecipe) {
	const background = backgroundOption(recipe.background)
	const symbol = symbolOption(recipe.symbol)
	if (!background || !symbol) return

	selectedBackground.value = background.id
	selectedSymbol.value = symbol.id
}

function surpriseMe() {
	const currentBackgroundIndex = backgroundOptions.findIndex(
		(option) => option.id === selectedBackground.value,
	)
	const backgroundOffset = Math.floor(Math.random() * (backgroundOptions.length - 1)) + 1
	selectedBackground.value =
		backgroundOptions[(currentBackgroundIndex + backgroundOffset) % backgroundOptions.length].id

	const currentSymbolIndex = symbolOptions.findIndex((option) => option.id === selectedSymbol.value)
	const symbolOffset = Math.floor(Math.random() * (symbolOptions.length - 1)) + 1
	selectedSymbol.value =
		symbolOptions[(currentSymbolIndex + symbolOffset) % symbolOptions.length].id
}

async function loadRecents() {
	try {
		recentRecipes.value = await get_recent_icon_recipes()
	} catch (error) {
		handleError(toError(error))
	}
}

function show() {
	selectedBackground.value = backgroundOption(props.recipe?.background)?.id ?? DEFAULT_BACKGROUND_ID
	selectedSymbol.value = symbolOption(props.recipe?.symbol ?? '')?.id ?? DEFAULT_SYMBOL_ID
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
		const recipe = selectedRecipe.value
		const symbolBytes = await loadSymbolBytes(selectedSymbolOption.value.asset)
		const iconPath = props.instanceId
			? await edit_generated_icon(props.instanceId, recipe, symbolBytes)
			: await cache_generated_icon(recipe, symbolBytes, true)
		emit('saved', iconPath, recipe)
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
		const recipe = selectedRecipe.value
		const iconPath = await cache_generated_icon(
			recipe,
			await loadSymbolBytes(selectedSymbolOption.value.asset),
		)
		return { iconPath, recipe }
	} catch (error) {
		handleError(toError(error))
		return null
	}
}

async function applyGeneratedIcon(instanceId: string, recipe: InstanceIconRecipe) {
	try {
		const symbol = symbolOption(recipe.symbol)
		if (!backgroundOption(recipe.background) || !symbol) return false

		await edit_generated_icon(instanceId, recipe, await loadSymbolBytes(symbol.asset))
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
					class="flex w-full flex-col items-center gap-3 rounded-[20px] border border-solid border-surface-4 bg-surface-2 p-4"
				>
					<div
						class="relative size-[132px] overflow-hidden rounded-[20px] border border-solid border-white/15"
						:style="backgroundStyle(selectedBackgroundOption.background)"
					>
						<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
					</div>
					<div class="flex items-center gap-2.5">
						<div
							class="relative size-12 overflow-hidden rounded-2xl border border-solid border-white/15"
							:style="backgroundStyle(selectedBackgroundOption.background)"
						>
							<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
						</div>
						<div
							class="relative size-8 overflow-hidden rounded-[10px] border border-solid border-white/15"
							:style="backgroundStyle(selectedBackgroundOption.background)"
						>
							<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
						</div>
						<div
							class="relative size-4 overflow-hidden rounded-[5px] border border-solid border-white/15"
							:style="backgroundStyle(selectedBackgroundOption.background)"
						>
							<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
						</div>
					</div>
				</div>

				<Button class="w-full !shadow-none" @click="surpriseMe">
					<RefreshCwIcon />
					{{ formatMessage(messages.surpriseMe) }}
				</Button>

				<div v-if="visibleRecentRecipes.length" class="flex flex-col gap-2.5">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.recents) }}</span>
					<div class="grid grid-cols-4 gap-3">
						<button
							v-for="(recentRecipe, index) in visibleRecentRecipes"
							:key="`${backgroundKey(recentRecipe.background)}-${recentRecipe.symbol}`"
							class="relative size-10 cursor-pointer overflow-hidden rounded-xl border border-solid border-white/15 p-0 transition-transform hover:scale-105"
							:style="backgroundStyle(recentRecipe.background)"
							:aria-label="`${formatMessage(messages.recents)} ${index + 1}`"
							@click="selectRecent(recentRecipe)"
						>
							<img
								:src="symbolOption(recentRecipe.symbol)?.asset"
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
							class="flex w-full gap-2.5 overflow-x-auto overflow-y-hidden pb-2 pr-6"
							@wheel.prevent="onBackgroundWheel"
							@scroll="updateBackgroundScrollShadows"
						>
							<button
								v-for="option in backgroundOptions"
								:key="option.id"
								class="relative aspect-square w-[calc((100%_-_3.125rem)/6)] shrink-0 cursor-pointer rounded-[20px] border border-solid p-0"
								:class="selectedBackground === option.id ? 'border-white/60' : 'border-white/15'"
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
						<button
							v-for="option in symbolOptions"
							:key="option.id"
							v-tooltip="{
								content: formatMessage(option.name),
								delay: { show: 500, hide: 0 },
							}"
							class="relative aspect-square cursor-pointer overflow-hidden rounded-[20px] border border-solid bg-transparent p-0"
							:class="selectedSymbol === option.id ? 'border-white/60' : 'border-white/15'"
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
					<Button :disabled="saving" @click="hide">
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
.background-scroll-shadow-left {
	-webkit-mask-image: linear-gradient(to right, black, transparent);
	mask-image: linear-gradient(to right, black, transparent);
}

.background-scroll-shadow-right {
	-webkit-mask-image: linear-gradient(to left, black, transparent);
	mask-image: linear-gradient(to left, black, transparent);
}
</style>
