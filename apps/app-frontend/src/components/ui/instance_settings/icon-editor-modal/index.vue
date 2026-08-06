<script setup lang="ts">
import { CheckIcon, InfoIcon, RefreshCwIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, ref } from 'vue'

import { toError } from '@/helpers/errors'
import { edit_generated_icon, get_recent_icon_recipes } from '@/helpers/instance'
import type { IconBackground, InstanceIconRecipe } from '@/helpers/types'

import {
	type BackgroundColor,
	backgroundOptions,
	DEFAULT_BACKGROUND_COLOR,
	DEFAULT_SYMBOL_ID,
	type SymbolId,
	symbolOptions,
} from './editor-catalog'

const props = defineProps<{
	instanceId: string
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

const selectedBackground = ref<BackgroundColor>(DEFAULT_BACKGROUND_COLOR)
const selectedSymbol = ref<SymbolId>(DEFAULT_SYMBOL_ID)

const selectedBackgroundOption = computed(
	() => backgroundOptions.find((option) => option.background.value === selectedBackground.value)!,
)
const selectedSymbolOption = computed(
	() => symbolOptions.find((option) => option.id === selectedSymbol.value)!,
)
const selectedBackgroundColor = computed(() => selectedBackgroundOption.value.background.value)
const selectedRecipe = computed<InstanceIconRecipe>(() => ({
	background: { type: 'color', value: selectedBackground.value },
	symbol: selectedSymbol.value,
}))
const visibleRecentRecipes = computed(() =>
	recentRecipes.value.filter(
		(recipe) => backgroundOption(recipe.background) && symbolOption(recipe.symbol),
	),
)

function backgroundOption(background?: IconBackground) {
	if (background?.type !== 'color') return undefined
	return backgroundOptions.find((option) => option.background.value === background.value)
}

function symbolOption(symbol: string) {
	return symbolOptions.find((option) => option.id === symbol)
}

function selectRecent(recipe: InstanceIconRecipe) {
	const background = backgroundOption(recipe.background)
	const symbol = symbolOption(recipe.symbol)
	if (!background || !symbol) return

	selectedBackground.value = background.background.value
	selectedSymbol.value = symbol.id
}

function surpriseMe() {
	const currentBackgroundIndex = backgroundOptions.findIndex(
		(option) => option.background.value === selectedBackground.value,
	)
	const backgroundOffset = Math.floor(Math.random() * (backgroundOptions.length - 1)) + 1
	selectedBackground.value =
		backgroundOptions[
			(currentBackgroundIndex + backgroundOffset) % backgroundOptions.length
		].background.value

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
	selectedBackground.value =
		backgroundOption(props.recipe?.background)?.background.value ?? DEFAULT_BACKGROUND_COLOR
	selectedSymbol.value = symbolOption(props.recipe?.symbol ?? '')?.id ?? DEFAULT_SYMBOL_ID
	modal.value?.show()
	void loadRecents()
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
		const iconPath = await edit_generated_icon(
			props.instanceId,
			recipe,
			await loadSymbolBytes(selectedSymbolOption.value.asset),
		)
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

defineExpose({ show, hide })

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
		defaultMessage: 'Surprise me',
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
						:style="{ backgroundColor: selectedBackgroundColor }"
					>
						<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
					</div>
					<div class="flex items-center gap-2.5">
						<div
							class="relative size-12 overflow-hidden rounded-2xl border border-solid border-white/15"
							:style="{ backgroundColor: selectedBackgroundColor }"
						>
							<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
						</div>
						<div
							class="relative size-8 overflow-hidden rounded-[10px] border border-solid border-white/15"
							:style="{ backgroundColor: selectedBackgroundColor }"
						>
							<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
						</div>
						<div
							class="relative size-4 overflow-hidden rounded-[5px] border border-solid border-white/15"
							:style="{ backgroundColor: selectedBackgroundColor }"
						>
							<img :src="selectedSymbolOption.asset" alt="" class="size-full object-cover" />
						</div>
					</div>
					<div class="flex items-center gap-2 text-sm font-medium text-primary">
						<span>{{ formatMessage(selectedBackgroundOption.name) }}</span>
						<span class="size-1.5 rounded-full bg-surface-5" />
						<span>{{ formatMessage(selectedSymbolOption.name) }}</span>
					</div>
				</div>

				<ButtonStyled class="w-full">
					<button class="w-full !shadow-none" @click="surpriseMe">
						<RefreshCwIcon />
						{{ formatMessage(messages.surpriseMe) }}
					</button>
				</ButtonStyled>

				<div v-if="visibleRecentRecipes.length" class="flex flex-col gap-2.5">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.recents) }}</span>
					<div class="grid grid-cols-4 gap-3">
						<button
							v-for="(recentRecipe, index) in visibleRecentRecipes"
							:key="`${recentRecipe.background.type}-${recentRecipe.background.value}-${recentRecipe.symbol}`"
							class="relative size-10 cursor-pointer overflow-hidden rounded-xl border border-solid border-white/15 p-0 transition-transform hover:scale-105"
							:style="{
								backgroundColor: backgroundOption(recentRecipe.background)?.background.value,
							}"
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
					<div class="grid grid-cols-6 gap-2.5">
						<button
							v-for="option in backgroundOptions"
							:key="option.background.value"
							class="relative aspect-square cursor-pointer rounded-[20px] border border-solid p-0"
							:class="
								selectedBackground === option.background.value
									? 'border-white/60'
									: 'border-white/15'
							"
							:style="{ backgroundColor: option.background.value }"
							:aria-label="formatMessage(option.name)"
							:aria-pressed="selectedBackground === option.background.value"
							@click="selectedBackground = option.background.value"
						>
							<span
								v-if="selectedBackground === option.background.value"
								class="absolute right-1.5 top-1.5 flex size-6 items-center justify-center rounded-full bg-white/80 text-black"
							>
								<CheckIcon class="size-4" />
							</span>
						</button>
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
					<ButtonStyled>
						<button :disabled="saving" @click="hide">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="saving" @click="saveIcon">
							<SpinnerIcon v-if="saving" class="animate-spin" />
							<SaveIcon v-else />
							{{ formatMessage(messages.saveIcon) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>
