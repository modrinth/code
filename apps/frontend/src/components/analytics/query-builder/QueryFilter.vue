<template>
	<div class="flex flex-wrap items-center gap-2">
		<span class="text-base font-medium text-primary">Filtered by</span>

		<Menu
			v-for="preview in appliedFilterPreviews"
			:key="preview.key"
			:triggers="['click']"
			:auto-hide="false"
			:popper-triggers="[]"
			no-auto-focus
			placement="bottom-start"
		>
			<div
				role="button"
				tabindex="0"
				class="inline-flex max-w-[16rem] items-center gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-1.5 text-sm text-primary transition-colors hover:bg-surface-4"
			>
				<span class="truncate">
					<span class="font-medium">{{ preview.label }}:</span>
					<span class="ml-1 font-semibold text-contrast">{{ preview.summary }}</span>
				</span>
				<ChevronDownIcon class="size-4 shrink-0 text-secondary" />
				<button
					type="button"
					class="-mr-1 inline-flex size-5 shrink-0 items-center justify-center rounded-full text-secondary transition-colors hover:bg-surface-5 hover:text-contrast"
					:aria-label="`Clear ${preview.label} filter`"
					@click.stop="clearFilterCategory(preview.key)"
				>
					<XIcon class="size-4" />
				</button>
			</div>

			<template #popper>
				<div>
					<p class="mb-2 text-xs font-semibold uppercase tracking-wide text-secondary">
						{{ preview.category.label }}
					</p>
					<div class="flex max-h-[min(70vh,32rem)] flex-col gap-2 overflow-y-auto pr-1">
						<Checkbox
							v-for="option in preview.category.options"
							:key="`${preview.category.key}-${option.value}`"
							:model-value="isFilterValueSelected(preview.category.key, option.value)"
							:label="option.label"
							@update:model-value="
								(nextValue) => toggleFilterValue(preview.category.key, option.value, nextValue)
							"
						/>
					</div>
				</div>
			</template>
		</Menu>

		<Menu
			v-model:shown="isAddMenuOpen"
			:triggers="['click']"
			:auto-hide="false"
			:popper-triggers="[]"
			no-auto-focus
			placement="bottom-start"
			@hide="resetPendingCategory"
		>
			<button
				ref="addMenuTrigger"
				type="button"
				class="inline-flex items-center gap-2 rounded-xl border border-dashed border-surface-5 bg-surface-2 px-3 py-1.5 text-sm font-semibold text-primary transition-colors hover:bg-surface-4"
			>
				<PlusIcon class="size-5" />
				Add
			</button>

			<template #popper>
				<div
					ref="menuContainer"
					class="flex w-[16rem] flex-col gap-1"
					@mousemove="(event) => handleMenuMouseMove(event, 'menu')"
				>
					<button
						v-for="category in filterCategories"
						:key="category.key"
						:ref="(element) => setCategoryButtonRef(category.key, element)"
						type="button"
						class="flex w-full appearance-none items-center justify-between rounded-lg border-0 px-2 py-1.5 text-left text-base font-medium text-primary shadow-none transition-colors hover:bg-surface-4"
						:class="category.key === activeCategoryKey ? 'bg-surface-4' : 'bg-transparent'"
						@mouseenter="handleCategoryMouseEnter(category.key)"
						@focus="activateCategory(category.key)"
					>
						<span>{{ category.label }}</span>
						<div class="flex items-center gap-1">
							<span
								v-if="getCategorySelectionCount(category.key) > 0"
								class="rounded-full bg-surface-5 px-1.5 py-0.5 text-xs text-primary"
							>
								{{ getCategorySelectionCount(category.key) }}
							</span>
							<ChevronRightIcon class="size-5 text-secondary" />
						</div>
					</button>
				</div>
			</template>
		</Menu>

		<Teleport to="body">
			<div
				v-if="isAddMenuOpen && activeCategory"
				ref="submenu"
				class="fixed z-[10000] min-w-[16rem] rounded-xl border border-solid border-surface-5 bg-surface-3 p-3 shadow-xl"
				:style="submenuStyle"
				@mousemove="(event) => handleMenuMouseMove(event, 'submenu')"
			>
				<p class="mb-2 text-xs font-semibold uppercase tracking-wide text-secondary">
					{{ activeCategory.label }}
				</p>
				<div class="flex max-h-[min(70vh,32rem)] flex-col gap-2 overflow-y-auto pr-1">
					<Checkbox
						v-for="option in activeCategory.options"
						:key="`${activeCategory.key}-${option.value}`"
						:model-value="isFilterValueSelected(activeCategory.key, option.value)"
						:label="option.label"
						@update:model-value="
							(nextValue) => toggleFilterValue(activeCategory.key, option.value, nextValue)
						"
					/>
				</div>
			</div>
		</Teleport>

		<button
			v-if="hasAppliedFilters"
			type="button"
			class="px-1.5 py-1 text-sm font-medium text-secondary transition-colors hover:text-contrast"
			@click="clearAllFilters"
		>
			Clear
		</button>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronRightIcon, PlusIcon, XIcon } from '@modrinth/assets'
import { Checkbox } from '@modrinth/ui'
import { Menu } from 'floating-vue'
import type { ComponentPublicInstance, CSSProperties } from 'vue'

import {
	type AnalyticsQueryFilterCategory,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

const ALL_FILTER_VALUE = '__all__'

type FilterOption = {
	value: string
	label: string
}

type FilterCategory = {
	key: AnalyticsQueryFilterCategory
	label: string
	allLabel: string
	options: FilterOption[]
}

type Point = {
	x: number
	y: number
}

const { projects, selectedProjectIds, selectedFilters } = injectAnalyticsDashboardContext()

const isAddMenuOpen = ref(false)
const activeCategoryKey = ref<AnalyticsQueryFilterCategory>('project')
const pendingCategoryKey = ref<AnalyticsQueryFilterCategory | null>(null)
const lastMousePosition = ref<Point | null>(null)
const addMenuTrigger = ref<HTMLElement | null>(null)
const menuContainer = ref<HTMLElement | null>(null)
const submenu = ref<HTMLElement | null>(null)
const submenuPosition = ref<Point>({ x: 0, y: 0 })
const categoryButtonRefs = new Map<AnalyticsQueryFilterCategory, HTMLElement>()
let pendingCategoryTimeout: ReturnType<typeof window.setTimeout> | null = null
let previousMousePosition: Point | null = null

const filterCategories = computed<FilterCategory[]>(() => [
	{
		key: 'project',
		label: 'Project',
		allLabel: 'All projects',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All projects' },
			...projects.value.map((project) => ({ value: project.id, label: project.name })),
		],
	},
	{
		key: 'country',
		label: 'Country',
		allLabel: 'All countries',
		options: withSelectedOptions('country', [{ value: ALL_FILTER_VALUE, label: 'All countries' }]),
	},
	{
		key: 'monetization',
		label: 'Monetization',
		allLabel: 'All',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All' },
			{ value: 'monetized', label: 'Monetized' },
			{ value: 'unmonetized', label: 'Unmonetized' },
		],
	},
	{
		key: 'download_source',
		label: 'Download Source',
		allLabel: 'All download sources',
		options: withSelectedOptions('download_source', [
			{ value: ALL_FILTER_VALUE, label: 'All download sources' },
		]),
	},
	{
		key: 'download_type',
		label: 'Download Type',
		allLabel: 'All download types',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All download types' },
			{ value: 'modpack', label: 'Modpack' },
			{ value: 'standalone', label: 'Standalone' },
			{ value: 'dependency', label: 'Dependency' },
		],
	},
	{
		key: 'game_version',
		label: 'Game Version',
		allLabel: 'All game versions',
		options: withSelectedOptions('game_version', [
			{ value: ALL_FILTER_VALUE, label: 'All game versions' },
		]),
	},
	{
		key: 'loader_type',
		label: 'Loader Type',
		allLabel: 'All loader types',
		options: withSelectedOptions('loader_type', [
			{ value: ALL_FILTER_VALUE, label: 'All loader types' },
		]),
	},
])

const filterCategoriesByKey = computed(
	() => new Map(filterCategories.value.map((category) => [category.key, category] as const)),
)

const activeCategory = computed(
	() => filterCategoriesByKey.value.get(activeCategoryKey.value) ?? filterCategories.value[0],
)

const submenuStyle = computed<CSSProperties>(() => ({
	left: `${submenuPosition.value.x}px`,
	top: `${submenuPosition.value.y}px`,
}))

const appliedFilterPreviews = computed(() =>
	filterCategories.value
		.map((category) => ({
			key: category.key,
			label: category.label,
			summary: getCategorySelectionSummary(category),
			count: getCategorySelectionCount(category.key),
			category,
		}))
		.filter((preview) => preview.count > 0),
)

const hasAppliedFilters = computed(() => appliedFilterPreviews.value.length > 0)

function setCategoryButtonRef(
	categoryKey: AnalyticsQueryFilterCategory,
	element: Element | ComponentPublicInstance | null,
) {
	if (element instanceof HTMLElement) {
		categoryButtonRefs.set(categoryKey, element)
	} else {
		categoryButtonRefs.delete(categoryKey)
	}
}

function withSelectedOptions(
	categoryKey: AnalyticsQueryFilterCategory,
	options: FilterOption[],
): FilterOption[] {
	const knownValues = new Set(options.map((option) => option.value))
	const selectedValues =
		categoryKey === 'project' ? selectedProjectIds.value : selectedFilters.value[categoryKey]
	const missingSelectedOptions = selectedValues
		.filter((value) => !knownValues.has(value))
		.map((value) => ({
			value,
			label: value,
		}))

	return [...options, ...missingSelectedOptions]
}

function isFilterValueSelected(categoryKey: AnalyticsQueryFilterCategory, value: string): boolean {
	if (categoryKey === 'project') {
		if (value === ALL_FILTER_VALUE) {
			return selectedProjectIds.value.length === projects.value.length
		}

		return selectedProjectIds.value.includes(value)
	}

	const values = selectedFilters.value[categoryKey]
	if (value === ALL_FILTER_VALUE) {
		return values.length === 0
	}
	return values.includes(value)
}

function toggleFilterValue(
	categoryKey: AnalyticsQueryFilterCategory,
	value: string,
	nextValue: boolean,
) {
	if (categoryKey === 'project') {
		if (value === ALL_FILTER_VALUE) {
			if (nextValue) {
				selectedProjectIds.value = projects.value.map((project) => project.id)
			}
			return
		}

		if (nextValue) {
			if (!selectedProjectIds.value.includes(value)) {
				selectedProjectIds.value = [...selectedProjectIds.value, value]
			}
			return
		}

		selectedProjectIds.value = selectedProjectIds.value.filter((projectId) => projectId !== value)
		if (selectedProjectIds.value.length === 0) {
			selectedProjectIds.value = projects.value.map((project) => project.id)
		}
		return
	}

	const currentValues = selectedFilters.value[categoryKey]

	if (value === ALL_FILTER_VALUE) {
		if (nextValue) {
			selectedFilters.value[categoryKey] = []
		}
		return
	}

	if (nextValue) {
		if (!currentValues.includes(value)) {
			selectedFilters.value[categoryKey] = [...currentValues, value]
		}
	} else {
		selectedFilters.value[categoryKey] = currentValues.filter((item) => item !== value)
	}
}

function getCategorySelectionCount(categoryKey: AnalyticsQueryFilterCategory): number {
	if (categoryKey === 'project') {
		return selectedProjectIds.value.length === projects.value.length
			? 0
			: selectedProjectIds.value.length
	}

	return selectedFilters.value[categoryKey].length
}

function getCategorySelectionSummary(category: FilterCategory): string {
	const count = getCategorySelectionCount(category.key)
	if (count === 0) {
		return category.allLabel
	}

	if (count === 1) {
		const selectedValue =
			category.key === 'project'
				? selectedProjectIds.value.find((projectId) =>
						projects.value.some((project) => project.id === projectId),
					)
				: selectedFilters.value[category.key][0]
		return category.options.find((option) => option.value === selectedValue)?.label ?? '1 selected'
	}

	return `${count} selected`
}

function clearFilterCategory(categoryKey: AnalyticsQueryFilterCategory) {
	if (categoryKey === 'project') {
		selectedProjectIds.value = projects.value.map((project) => project.id)
		return
	}

	selectedFilters.value[categoryKey] = []
}

function clearAllFilters() {
	selectedProjectIds.value = projects.value.map((project) => project.id)
	for (const category of filterCategories.value) {
		if (category.key !== 'project') {
			selectedFilters.value[category.key] = []
		}
	}
}

function activateCategory(categoryKey: AnalyticsQueryFilterCategory) {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	activeCategoryKey.value = categoryKey
	nextTick(updateSubmenuPosition)
}

function handleCategoryMouseEnter(categoryKey: AnalyticsQueryFilterCategory) {
	if (categoryKey === activeCategoryKey.value) {
		clearPendingCategoryTimeout()
		pendingCategoryKey.value = null
		return
	}

	if (!isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)) {
		activateCategory(categoryKey)
		return
	}

	pendingCategoryKey.value = categoryKey
	clearPendingCategoryTimeout()
	pendingCategoryTimeout = window.setTimeout(() => {
		if (pendingCategoryKey.value === categoryKey) {
			activateCategory(categoryKey)
		}
	}, 180)
}

function handleMenuMouseMove(event: MouseEvent, source: 'menu' | 'submenu') {
	previousMousePosition = lastMousePosition.value
	lastMousePosition.value = {
		x: event.clientX,
		y: event.clientY,
	}

	if (source !== 'menu') {
		return
	}

	if (
		!pendingCategoryKey.value ||
		isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)
	) {
		return
	}

	activateCategory(pendingCategoryKey.value)
}

function isCursorAimingAtSubmenu(cursor: Point | null, origin: Point | null): boolean {
	if (!submenu.value || !cursor || !origin) {
		return false
	}

	const submenuRect = submenu.value.getBoundingClientRect()
	const upperTarget: Point = {
		x: submenuRect.left,
		y: submenuRect.top,
	}
	const lowerTarget: Point = {
		x: submenuRect.left,
		y: submenuRect.bottom,
	}

	return isPointInTriangle(cursor, origin, upperTarget, lowerTarget)
}

function isPointInTriangle(point: Point, a: Point, b: Point, c: Point): boolean {
	const area = triangleArea(a, b, c)
	const area1 = triangleArea(point, b, c)
	const area2 = triangleArea(a, point, c)
	const area3 = triangleArea(a, b, point)

	return Math.abs(area - (area1 + area2 + area3)) < 0.5
}

function triangleArea(a: Point, b: Point, c: Point): number {
	return Math.abs((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2)
}

function clearPendingCategoryTimeout() {
	if (pendingCategoryTimeout) {
		window.clearTimeout(pendingCategoryTimeout)
		pendingCategoryTimeout = null
	}
}

function updateSubmenuPosition() {
	if (typeof window === 'undefined') {
		return
	}

	const activeButton = categoryButtonRefs.get(activeCategoryKey.value)
	if (!activeButton) {
		return
	}

	const buttonRect = activeButton.getBoundingClientRect()
	const submenuRect = submenu.value?.getBoundingClientRect()
	const submenuWidth = submenuRect?.width ?? 256
	const submenuHeight = submenuRect?.height ?? 320
	const gap = 8
	const viewportPadding = 8
	const preferredLeft = buttonRect.right + gap
	const left =
		preferredLeft + submenuWidth + viewportPadding <= window.innerWidth
			? preferredLeft
			: Math.max(viewportPadding, buttonRect.left - submenuWidth - gap)
	const top = Math.min(
		Math.max(viewportPadding, buttonRect.top),
		Math.max(viewportPadding, window.innerHeight - submenuHeight - viewportPadding),
	)

	submenuPosition.value = {
		x: left,
		y: top,
	}
}

function resetPendingCategory() {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	lastMousePosition.value = null
	previousMousePosition = null
}

function handleDocumentPointerDown(event: PointerEvent) {
	if (!isAddMenuOpen.value || !(event.target instanceof Node)) {
		return
	}

	if (
		addMenuTrigger.value?.contains(event.target) ||
		menuContainer.value?.contains(event.target) ||
		submenu.value?.contains(event.target)
	) {
		return
	}

	isAddMenuOpen.value = false
}

watch(isAddMenuOpen, (isOpen) => {
	if (isOpen && !activeCategory.value) {
		activeCategoryKey.value = filterCategories.value[0]?.key ?? 'project'
	}

	if (isOpen) {
		nextTick(updateSubmenuPosition)
		window.addEventListener('resize', updateSubmenuPosition)
		window.addEventListener('scroll', updateSubmenuPosition, true)
		window.addEventListener('pointerdown', handleDocumentPointerDown)
	} else {
		window.removeEventListener('resize', updateSubmenuPosition)
		window.removeEventListener('scroll', updateSubmenuPosition, true)
		window.removeEventListener('pointerdown', handleDocumentPointerDown)
	}
})

onBeforeUnmount(() => {
	clearPendingCategoryTimeout()
	if (typeof window !== 'undefined') {
		window.removeEventListener('resize', updateSubmenuPosition)
		window.removeEventListener('scroll', updateSubmenuPosition, true)
		window.removeEventListener('pointerdown', handleDocumentPointerDown)
	}
})
</script>
