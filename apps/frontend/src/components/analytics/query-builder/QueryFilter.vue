<template>
	<div class="flex flex-wrap items-center gap-2">
		<span class="text-base font-medium text-primary">Filtered by</span>

		<MultiSelect
			v-for="preview in appliedFilterPreviews"
			:key="preview.key"
			:model-value="getPreviewSelectedValues(preview.key)"
			:options="preview.options"
			:max-height="500"
			:clearable="false"
			:show-chevron="false"
			:fit-content="true"
			:trigger-class="previewTriggerClass"
			@update:model-value="(nextValue) => setPreviewSelectedValues(preview.key, nextValue)"
		>
			<template #input-content="{ isOpen }">
				<div class="flex min-w-0 items-center gap-2">
					<span class="truncate">
						<span class="font-medium">{{ preview.label }}:</span>
						<span class="ml-1 font-semibold text-contrast">{{ preview.summary }}</span>
					</span>
					<ChevronDownIcon
						class="size-4 shrink-0 text-secondary transition-transform duration-150"
						:class="isOpen ? 'rotate-180' : ''"
					/>
					<button
						type="button"
						class="-mr-1 inline-flex size-5 shrink-0 items-center justify-center rounded-full border-0 bg-transparent text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
						:aria-label="`Clear ${preview.label} filter`"
						@click.stop="clearFilterCategory(preview.key)"
					>
						<XIcon class="size-4" />
					</button>
				</div>
			</template>
		</MultiSelect>

		<Menu
			v-model:shown="isAddMenuOpen"
			:triggers="['click']"
			:delay="0"
			:auto-hide="false"
			:popper-triggers="[]"
			no-auto-focus
			placement="bottom-start"
			@hide="resetPendingCategory"
		>
			<button
				ref="addMenuTrigger"
				type="button"
				class="inline-flex h-10 items-center gap-2 rounded-xl border border-dashed border-surface-5 bg-surface-2 px-3 py-1.5 text-sm font-semibold text-primary transition-colors hover:bg-surface-4"
			>
				<PlusIcon class="size-5" />
				Add
			</button>

			<template #popper>
				<div
					ref="menuContainer"
					class="flex w-[12rem] flex-col gap-1"
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
				v-if="isAddMenuOpen && activeCategory && hasSubmenuPosition"
				ref="submenu"
				class="fixed z-[10000] min-w-[16rem] rounded-xl border border-solid border-surface-5 bg-surface-3 p-3 shadow-xl"
				:style="submenuStyle"
				@mouseenter="handleSubmenuMouseEnter"
				@mouseleave="handleSubmenuMouseLeave"
				@mousemove="(event) => handleMenuMouseMove(event, 'submenu')"
			>
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
			class="border-0 bg-transparent px-1.5 py-1 text-sm font-medium text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
			@click="clearAllFilters"
		>
			Clear
		</button>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronRightIcon, PlusIcon, XIcon } from '@modrinth/assets'
import { Checkbox, MultiSelect, type MultiSelectOption } from '@modrinth/ui'
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
const activeCategoryKey = ref<AnalyticsQueryFilterCategory | null>(null)
const pendingCategoryKey = ref<AnalyticsQueryFilterCategory | null>(null)
const lastMousePosition = ref<Point | null>(null)
const isCursorInsideSubmenu = ref(false)
const hasSubmenuPosition = ref(false)
const addMenuTrigger = ref<HTMLElement | null>(null)
const menuContainer = ref<HTMLElement | null>(null)
const submenu = ref<HTMLElement | null>(null)
const submenuPosition = ref<Point>({ x: 0, y: 0 })
const categoryButtonRefs = new Map<AnalyticsQueryFilterCategory, HTMLElement>()
let pendingCategoryTimeout: NodeJS.Timeout | null = null
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

const activeCategory = computed(() =>
	activeCategoryKey.value ? filterCategoriesByKey.value.get(activeCategoryKey.value) : undefined,
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
			options: category.options.map((option) => ({
				value: option.value,
				label: option.label,
			})) as MultiSelectOption<string>[],
		}))
		.filter((preview) => preview.count > 0),
)

const hasAppliedFilters = computed(() => appliedFilterPreviews.value.length > 0)
const previewTriggerClass =
	'h-10 max-w-[16rem] border border-solid border-surface-5 bg-surface-4 px-3 py-1.5 hover:bg-surface-5 hover:brightness-100 active:brightness-100'

function setCategoryButtonRef(
	categoryKey: AnalyticsQueryFilterCategory,
	element: Element | ComponentPublicInstance | null,
) {
	if (element instanceof HTMLElement) {
		categoryButtonRefs.set(categoryKey, element)
		if (isAddMenuOpen.value && categoryKey === activeCategoryKey.value) {
			scheduleSubmenuPositionUpdate()
		}
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

function getPreviewSelectedValues(categoryKey: AnalyticsQueryFilterCategory): string[] {
	if (categoryKey === 'project') {
		return selectedProjectIds.value
	}

	return selectedFilters.value[categoryKey]
}

function setPreviewSelectedValues(categoryKey: AnalyticsQueryFilterCategory, values: string[]) {
	if (categoryKey === 'project') {
		if (values.includes(ALL_FILTER_VALUE)) {
			selectedProjectIds.value = projects.value.map((project) => project.id)
			return
		}

		const allProjectIds = new Set(projects.value.map((project) => project.id))
		const selectedProjects = values.filter((value) => allProjectIds.has(value))

		selectedProjectIds.value =
			selectedProjects.length > 0 ? selectedProjects : projects.value.map((project) => project.id)
		return
	}

	if (values.includes(ALL_FILTER_VALUE) || values.length === 0) {
		selectedFilters.value[categoryKey] = []
		return
	}

	selectedFilters.value[categoryKey] = values.filter((value) => value !== ALL_FILTER_VALUE)
}

function activateCategory(categoryKey: AnalyticsQueryFilterCategory) {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	activeCategoryKey.value = categoryKey
	scheduleSubmenuPositionUpdate()
}

function handleCategoryMouseEnter(categoryKey: AnalyticsQueryFilterCategory) {
	if (!activeCategoryKey.value) {
		activateCategory(categoryKey)
		return
	}

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
	pendingCategoryTimeout = setTimeout(() => {
		if (pendingCategoryKey.value !== categoryKey) {
			return
		}

		if (
			isCursorInsideSubmenu.value ||
			isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)
		) {
			pendingCategoryKey.value = null
			return
		}

		activateCategory(categoryKey)
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
	const submenuTargetX =
		origin.x <= submenuRect.left
			? submenuRect.left
			: origin.x >= submenuRect.right
				? submenuRect.right
				: cursor.x <= submenuRect.left
					? submenuRect.left
					: submenuRect.right
	const upperTarget: Point = {
		x: submenuTargetX,
		y: submenuRect.top + 20,
	}
	const lowerTarget: Point = {
		x: submenuTargetX,
		y: submenuRect.bottom + 20,
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
		clearTimeout(pendingCategoryTimeout)
		pendingCategoryTimeout = null
	}
}

function updateSubmenuPosition(): boolean {
	if (typeof window === 'undefined') {
		return false
	}

	if (!activeCategoryKey.value) {
		return false
	}

	const activeButton = categoryButtonRefs.get(activeCategoryKey.value)
	if (!activeButton) {
		return false
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
	hasSubmenuPosition.value = true
	return true
}

function scheduleSubmenuPositionUpdate(retries = 8) {
	if (typeof window === 'undefined') {
		return
	}

	nextTick(() => {
		if (!isAddMenuOpen.value || updateSubmenuPosition() || retries <= 0) {
			return
		}

		window.requestAnimationFrame(() => scheduleSubmenuPositionUpdate(retries - 1))
	})
}

function resetPendingCategory() {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	isCursorInsideSubmenu.value = false
	lastMousePosition.value = null
	previousMousePosition = null
}

function handleSubmenuMouseEnter() {
	isCursorInsideSubmenu.value = true
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
}

function handleSubmenuMouseLeave() {
	isCursorInsideSubmenu.value = false
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
	if (isOpen) {
		activeCategoryKey.value = null
		hasSubmenuPosition.value = false
		scheduleSubmenuPositionUpdate()
		window.addEventListener('resize', updateSubmenuPosition)
		window.addEventListener('scroll', updateSubmenuPosition, true)
		window.addEventListener('pointerdown', handleDocumentPointerDown)
	} else {
		activeCategoryKey.value = null
		hasSubmenuPosition.value = false
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
