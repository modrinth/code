<script setup lang="ts">
import { DropdownIcon, EditIcon, PlusIcon, TrashIcon, UnknownIcon } from '@modrinth/assets'
import {
	Accordion,
	ButtonStyled,
	commonMessages,
	defineMessages,
	SkinButton,
	SkinLikeTextButton,
	useScrollViewport,
	useVIntl,
} from '@modrinth/ui'
import { useElementSize, useWindowSize } from '@vueuse/core'
import { Tooltip } from 'floating-vue'
import { computed, nextTick, onUnmounted, ref, useTemplateRef, watch } from 'vue'
import Draggable from 'vuedraggable'

import type { RenderResult } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Skin } from '@/helpers/skins.ts'

type SkinSectionKind = 'saved' | 'default'
type SkinLikeTextButtonExpose = {
	getRootElement: () => HTMLElement | null | undefined
}
type AddSkinButtonRef = SkinLikeTextButtonExpose | SkinLikeTextButtonExpose[]

interface DefaultSkinSection {
	title: string
	infoTooltip?: string
	skins: Skin[]
}

interface SkinSection {
	key: string
	title: string
	kind: SkinSectionKind
	infoTooltip?: string
	skins: Skin[]
}

interface VirtualSkinSection {
	section: SkinSection
	top: number
	index: number
}

const SKIN_CARD_ASPECT_WIDTH = 31
const SKIN_CARD_ASPECT_HEIGHT = 40
const SKIN_GRID_GAP = 12
const SKIN_SECTION_FIRST_SPACING = 4
const SKIN_SECTION_SPACING = 24
const SKIN_SECTION_HEADER_HEIGHT = 28
const SKIN_SECTION_CONTENT_SPACING = 8
const SKIN_SECTION_OVERSCAN = 900
const FALLBACK_CARD_WIDTH = 220
const messages = defineMessages({
	savedSkinsSection: {
		id: 'app.skins.section.saved-skins',
		defaultMessage: 'Saved skins',
	},
	addSkinButton: {
		id: 'app.skins.add-button',
		defaultMessage: 'Add skin',
	},
	dragAndDropSubtitle: {
		id: 'app.skins.add-button.drag-and-drop',
		defaultMessage: 'Drag and drop',
	},
	editSkinButton: {
		id: 'app.skins.edit-button',
		defaultMessage: 'Edit skin',
	},
	deleteSkinButton: {
		id: 'app.skins.delete-button',
		defaultMessage: 'Delete skin',
	},
})

const props = defineProps<{
	savedSkins: Skin[]
	defaultSkinSections: DefaultSkinSection[]
	getBakedSkinTextures: (skin: Skin) => RenderResult | undefined
	isSkinSelected: (skin: Skin) => boolean
	isSkinActive: (skin: Skin) => boolean
	isAddSkinButtonDragActive: boolean
	readOnly?: boolean
}>()

const emit = defineEmits<{
	select: [skin: Skin]
	edit: [skin: Skin, event: MouseEvent]
	delete: [skin: Skin]
	'reorder-saved-skins': [skins: Skin[]]
	'add-skin': []
	'add-skin-dragenter': [event: DragEvent]
	'add-skin-dragover': [event: DragEvent]
	'add-skin-dragleave': [event: DragEvent]
	'add-skin-drop': [event: DragEvent]
}>()

const addSkinButton = useTemplateRef<AddSkinButtonRef>('addSkinButton')
const { formatMessage } = useVIntl()
const { listContainer, relativeScrollTop, scrollContainer, viewportHeight } = useScrollViewport()
const openSectionKeys = ref<Set<string>>(new Set())
const hasSettledInitialLayout = ref(false)
const knownSectionKeys = new Set<string>()
let enableLayoutTransitionsFrame: number | null = null
let isEnableLayoutTransitionsScheduled = false
let isUnmounted = false

const { width: listWidth } = useElementSize(listContainer)
const { width: windowWidth } = useWindowSize()

const columnCount = computed(() => {
	if (windowWidth.value >= 2050) {
		return 6
	}

	if (windowWidth.value >= 1750) {
		return 5
	}

	if (windowWidth.value >= 1300) {
		return 4
	}

	return 3
})

const cardWidth = computed(() => {
	if (listWidth.value <= 0) {
		return FALLBACK_CARD_WIDTH
	}

	const gapsWidth = (columnCount.value - 1) * SKIN_GRID_GAP
	return Math.max(0, (listWidth.value - gapsWidth) / columnCount.value)
})

const cardHeight = computed(
	() => (cardWidth.value * SKIN_CARD_ASPECT_HEIGHT) / SKIN_CARD_ASPECT_WIDTH,
)

const sections = computed<SkinSection[]>(() => [
	{
		key: 'saved-skins',
		title: formatMessage(messages.savedSkinsSection),
		kind: 'saved',
		skins: props.savedSkins,
	},
	...props.defaultSkinSections.map((section) => ({
		key: defaultSkinSectionKey(section.title),
		title: section.title,
		kind: 'default' as const,
		infoTooltip: section.infoTooltip,
		skins: section.skins,
	})),
])

const draggableSavedSkins = ref<Skin[]>([])
const isDraggingSavedSkin = ref(false)
const canReorderSavedSkins = computed(() => draggableSavedSkins.value.length > 1)
const fixedSavedSkins = computed(() =>
	props.savedSkins.filter((skin) => !canDragSavedSkin(skin)),
)

const sectionLayouts = computed(() => {
	const layouts: Array<{ section: SkinSection; top: number; height: number; index: number }> = []
	let top = 0

	sections.value.forEach((section, index) => {
		const height = getSectionHeightEstimate(section, index)
		layouts.push({ section, top, height, index })
		top += height
	})

	return layouts
})

const totalHeight = computed(() => {
	const lastSection = sectionLayouts.value[sectionLayouts.value.length - 1]
	return lastSection ? lastSection.top + lastSection.height : 0
})

const visibleSections = computed<VirtualSkinSection[]>(() => {
	if (!listContainer.value || !scrollContainer.value) {
		return sectionLayouts.value.slice(0, 4)
	}

	const viewportStart = Math.max(0, relativeScrollTop.value - SKIN_SECTION_OVERSCAN)
	const viewportEnd = relativeScrollTop.value + viewportHeight.value + SKIN_SECTION_OVERSCAN

	return sectionLayouts.value
		.filter((layout) => layout.top + layout.height >= viewportStart && layout.top <= viewportEnd)
		.map(({ section, top, index }) => ({ section, top, index }))
})

watch(
	sections,
	(nextSections) => {
		const sectionKeys = new Set(nextSections.map((section) => section.key))
		const openKeys = new Set(openSectionKeys.value)

		for (const section of nextSections) {
			if (!knownSectionKeys.has(section.key)) {
				knownSectionKeys.add(section.key)
				openKeys.add(section.key)
			}
		}

		for (const key of knownSectionKeys) {
			if (!sectionKeys.has(key)) {
				knownSectionKeys.delete(key)
				openKeys.delete(key)
			}
		}

		openSectionKeys.value = openKeys
	},
	{ immediate: true },
)

watch(
	() => props.savedSkins,
	(nextSkins) => {
		if (isDraggingSavedSkin.value) {
			return
		}

		draggableSavedSkins.value = nextSkins.filter(canDragSavedSkin)
	},
	{ immediate: true },
)

watch(
	listWidth,
	(width) => {
		if (
			typeof window === 'undefined' ||
			width <= 0 ||
			hasSettledInitialLayout.value ||
			isEnableLayoutTransitionsScheduled
		) {
			return
		}

		isEnableLayoutTransitionsScheduled = true
		void nextTick(() => {
			if (isUnmounted) return

			enableLayoutTransitionsFrame = window.requestAnimationFrame(() => {
				if (isUnmounted) return

				enableLayoutTransitionsFrame = window.requestAnimationFrame(() => {
					if (isUnmounted) return

					hasSettledInitialLayout.value = true
					enableLayoutTransitionsFrame = null
					isEnableLayoutTransitionsScheduled = false
				})
			})
		})
	},
	{ immediate: true },
)

onUnmounted(() => {
	isUnmounted = true

	if (enableLayoutTransitionsFrame !== null) {
		window.cancelAnimationFrame(enableLayoutTransitionsFrame)
	}
})

function defaultSkinSectionKey(title: string) {
	return `default-skins-${title}`
}

function skinKey(skin: Skin, prefix: string) {
	return `${prefix}-${skin.source}-${skin.texture_key}-${skin.variant}-${skin.cape_id ?? 'no-cape'}`
}

function savedSkinKey(skin: Skin) {
	return skinKey(skin, 'saved-skin')
}

function canDragSavedSkin(skin: Skin) {
	return skin.source === 'custom' || skin.source === 'custom_external'
}

function doSkinOrdersMatch(firstSkins: Skin[], secondSkins: Skin[]) {
	const draggableSecondSkins = secondSkins.filter(canDragSavedSkin)

	return (
		firstSkins.length === draggableSecondSkins.length &&
		firstSkins.every(
			(skin, index) => savedSkinKey(skin) === savedSkinKey(draggableSecondSkins[index]),
		)
	)
}

function onSavedSkinDragStart() {
	isDraggingSavedSkin.value = true
}

function onSavedSkinDragEnd() {
	isDraggingSavedSkin.value = false

	if (doSkinOrdersMatch(draggableSavedSkins.value, props.savedSkins)) {
		draggableSavedSkins.value = props.savedSkins.filter(canDragSavedSkin)
		return
	}

	emit('reorder-saved-skins', [...draggableSavedSkins.value])
}

function isSectionOpen(key: string) {
	return openSectionKeys.value.has(key)
}

function setSectionOpen(key: string, open: boolean) {
	const openKeys = new Set(openSectionKeys.value)

	if (open) {
		openKeys.add(key)
	} else {
		openKeys.delete(key)
	}

	openSectionKeys.value = openKeys
}

function getSectionHeightEstimate(section: SkinSection, index: number) {
	const spacing = index === 0 ? SKIN_SECTION_FIRST_SPACING : SKIN_SECTION_SPACING

	if (!isSectionOpen(section.key)) {
		return spacing + SKIN_SECTION_HEADER_HEIGHT
	}

	const cardCount = section.kind === 'saved' ? section.skins.length + 1 : section.skins.length
	const rowCount = Math.ceil(cardCount / columnCount.value)
	const gridHeight = rowCount * cardHeight.value + Math.max(0, rowCount - 1) * SKIN_GRID_GAP

	return spacing + SKIN_SECTION_HEADER_HEIGHT + SKIN_SECTION_CONTENT_SPACING + gridHeight
}

function getAddSkinButtonElement() {
	const button = Array.isArray(addSkinButton.value)
		? addSkinButton.value.find((candidate) => candidate.getRootElement())
		: addSkinButton.value

	return button?.getRootElement()
}

defineExpose({ getAddSkinButtonElement })
</script>

<template>
	<div
		ref="listContainer"
		class="relative w-full"
		:style="{ height: `${totalHeight}px`, overflowAnchor: 'none' }"
	>
		<div
			v-for="{ section, top, index } in visibleSections"
			:key="section.key"
			class="absolute inset-x-0"
			:class="[
				index === 0 ? 'pt-1' : 'pt-6',
				hasSettledInitialLayout
					? 'transition-transform duration-300 ease-in-out will-change-transform motion-reduce:transition-none'
					: '',
			]"
			:style="{ transform: `translateY(${top}px)` }"
		>
			<Accordion
				button-class="group flex w-full items-center gap-[6px] bg-transparent m-0 p-0 border-none cursor-pointer text-left"
				content-class="pt-2"
				:open-by-default="isSectionOpen(section.key)"
				@on-open="setSectionOpen(section.key, true)"
				@on-close="setSectionOpen(section.key, false)"
			>
				<template #title>
					{{ section.title }}
				</template>
				<template #button="{ open }">
					<DropdownIcon
						class="size-6 shrink-0 text-primary transition-transform duration-300"
						:class="{ 'rotate-180': open }"
					/>
					<span class="min-w-0 text-xl font-semibold leading-7 text-primary">
						{{ section.title }}
					</span>
					<Tooltip
						v-if="section.infoTooltip"
						theme="dismissable-prompt"
						placement="top"
						:triggers="['hover', 'focus']"
					>
						<span
							class="inline-flex size-6 shrink-0 items-center justify-center text-secondary transition-colors group-hover:text-primary"
							@click.stop
						>
							<UnknownIcon class="size-5" />
						</span>
						<template #popper>
							<p class="m-0 max-w-96 text-wrap text-sm font-medium leading-tight">
								{{ section.infoTooltip }}
							</p>
						</template>
					</Tooltip>
				</template>

				<Draggable
					v-if="section.kind === 'saved'"
					:list="draggableSavedSkins"
					class="grid w-full grid-cols-3 gap-3 min-[1300px]:grid-cols-4 min-[1750px]:grid-cols-5 min-[2050px]:grid-cols-6"
					:item-key="savedSkinKey"
					:disabled="readOnly || !canReorderSavedSkins"
					:animation="250"
					:swap-threshold="1"
					:invert-swap="false"
					:force-fallback="true"
					:fallback-on-body="true"
					:fallback-tolerance="4"
					ghost-class="skin-reorder-ghost"
					chosen-class="skin-reorder-chosen"
					drag-class="skin-reorder-drag"
					fallback-class="skin-reorder-fallback"
					@start="onSavedSkinDragStart"
					@end="onSavedSkinDragEnd"
				>
					<template #header>
						<SkinLikeTextButton
							ref="addSkinButton"
							class="aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
							dropzone
							:disabled="readOnly"
							:drag-active="!readOnly && isAddSkinButtonDragActive"
							@click="emit('add-skin')"
							@dragenter="emit('add-skin-dragenter', $event)"
							@dragover="emit('add-skin-dragover', $event)"
							@dragleave="emit('add-skin-dragleave', $event)"
							@drop="emit('add-skin-drop', $event)"
						>
							<template #icon>
								<PlusIcon class="size-8" />
							</template>
							{{ formatMessage(messages.addSkinButton) }}
							<template #subtitle>{{ formatMessage(messages.dragAndDropSubtitle) }}</template>
						</SkinLikeTextButton>
					</template>

					<template #item="{ element: skin }">
						<div
							:key="savedSkinKey(skin)"
							class="relative aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
						>
							<SkinButton
								class="h-full w-full min-w-0 box-border rounded-[20px]"
								:forward-image-src="getBakedSkinTextures(skin)?.forwards"
								:selected="isSkinSelected(skin)"
								:active="isSkinActive(skin)"
								:disabled="readOnly"
								:is-dragging="isDraggingSavedSkin"
								@select="emit('select', skin)"
							>
								<template v-if="!readOnly" #overlay-buttons>
									<ButtonStyled color="brand">
										<button
											:aria-label="formatMessage(messages.editSkinButton)"
											class="pointer-events-auto"
											@click.stop="(event: MouseEvent) => emit('edit', skin, event)"
										>
											<EditIcon /> {{ formatMessage(commonMessages.editButton) }}
										</button>
									</ButtonStyled>
									<ButtonStyled v-show="!skin.is_equipped" circular color="red">
										<button
											v-tooltip="formatMessage(messages.deleteSkinButton)"
											:aria-label="formatMessage(messages.deleteSkinButton)"
											class="!rounded-[100%] pointer-events-auto"
											@click.stop="emit('delete', skin)"
										>
											<TrashIcon />
										</button>
									</ButtonStyled>
								</template>
							</SkinButton>
						</div>
					</template>

					<template #footer>
						<div
							v-for="skin in fixedSavedSkins"
							:key="savedSkinKey(skin)"
							class="relative aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
						>
							<SkinButton
								class="h-full w-full min-w-0 box-border rounded-[20px]"
								:forward-image-src="getBakedSkinTextures(skin)?.forwards"
								:selected="isSkinSelected(skin)"
								:active="isSkinActive(skin)"
								:disabled="readOnly"
								:is-dragging="isDraggingSavedSkin"
								@select="emit('select', skin)"
							>
								<template v-if="!readOnly" #overlay-buttons>
									<ButtonStyled color="brand">
										<button
											:aria-label="formatMessage(messages.editSkinButton)"
											class="pointer-events-auto"
											@click.stop="(event: MouseEvent) => emit('edit', skin, event)"
										>
											<EditIcon /> {{ formatMessage(commonMessages.editButton) }}
										</button>
									</ButtonStyled>
									<ButtonStyled v-show="!skin.is_equipped" circular color="red">
										<button
											v-tooltip="formatMessage(messages.deleteSkinButton)"
											:aria-label="formatMessage(messages.deleteSkinButton)"
											class="!rounded-[100%] pointer-events-auto"
											@click.stop="emit('delete', skin)"
										>
											<TrashIcon />
										</button>
									</ButtonStyled>
								</template>
							</SkinButton>
						</div>
					</template>
				</Draggable>

				<div
					v-else
					class="grid w-full grid-cols-3 gap-3 min-[1300px]:grid-cols-4 min-[1750px]:grid-cols-5 min-[2050px]:grid-cols-6"
				>
					<SkinButton
						v-for="skin in section.skins"
						:key="skinKey(skin, section.key)"
						class="aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
						:forward-image-src="getBakedSkinTextures(skin)?.forwards"
						:selected="isSkinSelected(skin)"
						:active="isSkinActive(skin)"
						:tooltip="skin.name"
						:disabled="readOnly"
						:is-dragging="isDraggingSavedSkin"
						@select="emit('select', skin)"
					>
						<template #overlay-buttons>
							<ButtonStyled color="brand">
								<button
									:aria-label="formatMessage(messages.editSkinButton)"
									class="pointer-events-auto"
									@click.stop="(event: MouseEvent) => emit('edit', skin, event)"
								>
									<EditIcon /> {{ formatMessage(commonMessages.editButton) }}
								</button>
							</ButtonStyled>
						</template>
					</SkinButton>
				</div>
			</Accordion>
		</div>
	</div>
</template>

<style scoped>
:global(.skin-reorder-ghost) {
	opacity: 0.35;
}

:global(.skin-reorder-drag) {
	cursor: grabbing;
}

:global(.skin-reorder-fallback) {
	opacity: 0.9;
	pointer-events: none;
}
</style>
