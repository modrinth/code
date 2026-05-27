<script setup lang="ts">
import { DropdownIcon, EditIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	Accordion,
	ButtonStyled,
	SkinButton,
	SkinLikeTextButton,
	useScrollViewport,
} from '@modrinth/ui'
import { useElementSize, useWindowSize } from '@vueuse/core'
import { computed, nextTick, onUnmounted, ref, useTemplateRef, watch } from 'vue'

import type { RenderResult } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Skin } from '@/helpers/skins.ts'

type SkinSectionKind = 'saved' | 'default'
type SkinLikeTextButtonExpose = {
	getRootElement: () => HTMLElement | null | undefined
}

interface DefaultSkinSection {
	title: string
	skins: Skin[]
}

interface SkinSection {
	key: string
	title: string
	kind: SkinSectionKind
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

const props = defineProps<{
	savedSkins: Skin[]
	defaultSkinSections: DefaultSkinSection[]
	getBakedSkinTextures: (skin: Skin) => RenderResult | undefined
	isSkinSelected: (skin: Skin) => boolean
	isAddSkinButtonDragActive: boolean
}>()

const emit = defineEmits<{
	select: [skin: Skin]
	edit: [skin: Skin, event: MouseEvent]
	delete: [skin: Skin]
	'add-skin': []
	'add-skin-dragenter': [event: DragEvent]
	'add-skin-dragover': [event: DragEvent]
	'add-skin-dragleave': [event: DragEvent]
	'add-skin-drop': [event: DragEvent]
}>()

const addSkinButton = useTemplateRef<SkinLikeTextButtonExpose>('addSkinButton')
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
		title: 'Saved skins',
		kind: 'saved',
		skins: props.savedSkins,
	},
	...props.defaultSkinSections.map((section) => ({
		key: defaultSkinSectionKey(section.title),
		title: section.title,
		kind: 'default' as const,
		skins: section.skins,
	})),
])

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
	return addSkinButton.value?.getRootElement()
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
				</template>

				<div
					v-if="section.kind === 'saved'"
					class="grid w-full grid-cols-3 gap-3 min-[1300px]:grid-cols-4 min-[1750px]:grid-cols-5 min-[2050px]:grid-cols-6"
				>
					<SkinLikeTextButton
						ref="addSkinButton"
						class="aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
						dropzone
						:drag-active="isAddSkinButtonDragActive"
						@click="emit('add-skin')"
						@dragenter="emit('add-skin-dragenter', $event)"
						@dragover="emit('add-skin-dragover', $event)"
						@dragleave="emit('add-skin-dragleave', $event)"
						@drop="emit('add-skin-drop', $event)"
					>
						<template #icon>
							<PlusIcon class="size-8" />
						</template>
						Add skin
						<template #subtitle>Drag and drop</template>
					</SkinLikeTextButton>

					<SkinButton
						v-for="skin in section.skins"
						:key="skinKey(skin, 'saved-skin')"
						class="aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
						:forward-image-src="getBakedSkinTextures(skin)?.forwards"
						:backward-image-src="getBakedSkinTextures(skin)?.backwards"
						:selected="isSkinSelected(skin)"
						@select="emit('select', skin)"
					>
						<template #overlay-buttons>
							<ButtonStyled color="brand">
								<button
									aria-label="Edit skin"
									class="pointer-events-auto"
									@click.stop="(event: MouseEvent) => emit('edit', skin, event)"
								>
									<EditIcon /> Edit
								</button>
							</ButtonStyled>
							<ButtonStyled v-show="!skin.is_equipped" circular color="red">
								<button
									v-tooltip="'Delete skin'"
									aria-label="Delete skin"
									class="!rounded-[100%] pointer-events-auto"
									@click.stop="emit('delete', skin)"
								>
									<TrashIcon />
								</button>
							</ButtonStyled>
						</template>
					</SkinButton>
				</div>

				<div
					v-else
					class="grid w-full grid-cols-3 gap-3 min-[1300px]:grid-cols-4 min-[1750px]:grid-cols-5 min-[2050px]:grid-cols-6"
				>
					<SkinButton
						v-for="skin in section.skins"
						:key="skinKey(skin, section.key)"
						class="aspect-[31/40] w-full min-w-0 box-border rounded-[20px]"
						:forward-image-src="getBakedSkinTextures(skin)?.forwards"
						:backward-image-src="getBakedSkinTextures(skin)?.backwards"
						:selected="isSkinSelected(skin)"
						:tooltip="skin.name"
						@select="emit('select', skin)"
					/>
				</div>
			</Accordion>
		</div>
	</div>
</template>
