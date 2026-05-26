import { BoxIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import DropdownFilterBar from '../../components/base/DropdownFilterBar.vue'

const meta = {
	title: 'Base/DropdownFilterBar',
	component: DropdownFilterBar,
} satisfies Meta<typeof DropdownFilterBar>

export default meta
type Story = StoryObj<typeof meta>

const defaultCategories = [
	{
		key: 'status',
		label: 'Status',
		options: [
			{ value: 'active', label: 'Active' },
			{ value: 'archived', label: 'Archived' },
			{ value: 'draft', label: 'Draft' },
		],
	},
	{
		key: 'type',
		label: 'Type',
		options: [
			{ value: 'mod', label: 'Mod' },
			{ value: 'plugin', label: 'Plugin' },
			{ value: 'resourcepack', label: 'Resource Pack' },
			{ value: 'modpack', label: 'Modpack' },
		],
	},
]

const searchableCategories = [
	{
		key: 'country',
		label: 'Country',
		searchable: true,
		searchPlaceholder: 'Search countries...',
		options: [
			{ value: 'US', label: 'United States', searchTerms: ['USA', 'America'] },
			{ value: 'CA', label: 'Canada' },
			{ value: 'DE', label: 'Germany', searchTerms: ['Deutschland'] },
			{ value: 'JP', label: 'Japan' },
			{ value: 'BR', label: 'Brazil' },
			{ value: 'AU', label: 'Australia' },
		],
	},
	{
		key: 'version',
		label: 'Version',
		searchable: true,
		searchPlaceholder: 'Search versions...',
		submenuClass: 'w-[360px]',
		options: [
			{ value: '1.21.5', label: '1.21.5', searchTerms: ['Sodium'] },
			{ value: '1.21.4', label: '1.21.4', searchTerms: ['Sodium'] },
			{ value: '1.20.1', label: '1.20.1', searchTerms: ['Iris'] },
			{ value: '1.19.2', label: '1.19.2', searchTerms: ['Mod Menu'] },
		],
	},
]

const largeVersionOptions = Array.from({ length: 250 }, (_, index) => {
	const version = `1.${Math.floor(index / 10) + 1}.${index % 10}`
	const project = `Project ${Math.floor(index / 25) + 1}`
	return {
		value: `version-${index + 1}`,
		label: version,
		searchTerms: [project],
	}
})

const mixedWidthCategories = [
	{
		key: 'status',
		label: 'Status',
		options: [
			{ value: 'active', label: 'Active' },
			{ value: 'archived', label: 'Archived' },
			{ value: 'draft', label: 'Draft' },
		],
	},
	{
		key: 'country',
		label: 'Country',
		searchable: true,
		searchPlaceholder: 'Search countries...',
		submenuClass: 'w-[324px]',
		options: [
			{ value: 'US', label: 'United States' },
			{ value: 'CA', label: 'Canada' },
			{ value: 'DE', label: 'Germany' },
			{ value: 'JP', label: 'Japan' },
		],
	},
	{
		key: 'version',
		label: 'Project version',
		searchable: true,
		searchPlaceholder: 'Search project versions...',
		submenuClass: 'w-[368px]',
		options: [
			{ value: 'sodium-1.21.5', label: 'Sodium 1.21.5' },
			{ value: 'iris-1.21.4', label: 'Iris 1.21.4' },
			{ value: 'mod-menu-1.20.1', label: 'Mod Menu 1.20.1' },
		],
	},
]

export const Default: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			return { categories: defaultCategories, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories" />
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: defaultCategories,
	},
}

export const WithAppliedFilters: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({
				status: ['active'],
				type: ['mod', 'plugin'],
			})
			const clearEvents = ref(0)
			function handleClear() {
				clearEvents.value += 1
			}

			return { categories: defaultCategories, clearEvents, handleClear, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar
					v-model="selected"
					:categories="categories"
					@clear="handleClear"
				/>
				<span class="text-sm font-medium text-secondary">Clear events: {{ clearEvents }}</span>
			</div>
		`,
	}),
	args: {
		modelValue: {
			status: ['active'],
			type: ['mod', 'plugin'],
		},
		categories: defaultCategories,
	},
}

export const WithRightCheckmarks: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({
				status: ['active'],
				type: ['mod', 'plugin'],
			})
			return { categories: defaultCategories, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar
					v-model="selected"
					:categories="categories"
					checkbox-position="right"
				/>
			</div>
		`,
	}),
	args: {
		modelValue: {
			status: ['active'],
			type: ['mod', 'plugin'],
		},
		categories: defaultCategories,
		checkboxPosition: 'right',
	},
	parameters: {
		docs: {
			description: {
				story:
					'Renders selected options with the same right-side checkmark placement as MultiSelect.',
			},
		},
	},
}

export const WithClearOverride: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			const clearEvents = ref(0)
			function handleClear() {
				clearEvents.value += 1
			}

			return { categories: defaultCategories, clearEvents, handleClear, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar
					v-model="selected"
					:categories="categories"
					show-clear
					@clear="handleClear"
				/>
				<span class="text-sm font-medium text-secondary">Clear events: {{ clearEvents }}</span>
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: defaultCategories,
		showClear: true,
	},
}

export const WithFilterIcon: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({
				status: ['draft'],
			})
			return { categories: defaultCategories, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories" use-filter-icon />
			</div>
		`,
	}),
	args: {
		modelValue: {
			status: ['draft'],
		},
		categories: defaultCategories,
		useFilterIcon: true,
	},
}

export const SearchableCategories: Story = {
	render: () => ({
		components: { BoxIcon, DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			const versionProjects: Record<string, string> = {
				'1.21.5': 'Sodium',
				'1.21.4': 'Sodium',
				'1.20.1': 'Iris',
				'1.19.2': 'Mod Menu',
			}
			function getVersionProject(categoryKey: string, optionValue: string) {
				return categoryKey === 'version' ? versionProjects[optionValue] : undefined
			}
			return { categories: searchableCategories, getVersionProject, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories">
					<template #option-right="{ category, option }">
						<span
							v-if="getVersionProject(category.key, option.value)"
							v-tooltip="getVersionProject(category.key, option.value)"
							class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
						>
							<BoxIcon class="size-6" />
						</span>
					</template>
				</DropdownFilterBar>
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: searchableCategories,
	},
	parameters: {
		docs: {
			description: {
				story:
					'On mobile and narrow viewports, tapping a category replaces the add menu with the category submenu; clicking outside either surface should close the add filter dropdown.',
			},
		},
	},
}

export const MixedSubmenuWidthsNearEdge: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			return { categories: mixedWidthCategories, selected }
		},
		template: /* html */ `
			<div class="flex min-h-96 justify-end px-4 py-8">
				<DropdownFilterBar v-model="selected" :categories="categories" />
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: mixedWidthCategories,
	},
	parameters: {
		docs: {
			description: {
				story:
					'Covers mixed submenu widths near the viewport edge so all add-menu submenus open on the same side.',
			},
		},
	},
}

export const VirtualizedPreview: Story = {
	render: () => ({
		components: { BoxIcon, DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({
				version: ['version-3', 'version-47', 'version-132'],
			})
			const categories = [
				{
					key: 'version',
					label: 'Version',
					searchable: true,
					searchPlaceholder: 'Search versions...',
					submenuClass: 'w-[360px]',
					previewDropdownWidth: '360px',
					options: largeVersionOptions,
				},
			]
			function getVersionProject(categoryKey: string, optionValue: string) {
				if (categoryKey !== 'version') {
					return undefined
				}
				const optionIndex = Number(optionValue.replace('version-', '')) - 1
				return `Project ${Math.floor(optionIndex / 25) + 1}`
			}
			return { categories, getVersionProject, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories">
					<template #option-right="{ category, option }">
						<span
							v-if="getVersionProject(category.key, option.value)"
							v-tooltip="getVersionProject(category.key, option.value)"
							class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
						>
							<BoxIcon class="size-6" />
						</span>
					</template>
				</DropdownFilterBar>
			</div>
		`,
	}),
	args: {
		modelValue: {
			version: ['version-3', 'version-47', 'version-132'],
		},
		categories: [
			{
				key: 'version',
				label: 'Version',
				searchable: true,
				searchPlaceholder: 'Search versions...',
				submenuClass: 'w-[360px]',
				previewDropdownWidth: '360px',
				options: largeVersionOptions,
			},
		],
	},
}

export const VirtualizedSubmenu: Story = {
	render: () => ({
		components: { BoxIcon, DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			const categories = [
				{
					key: 'version',
					label: 'Version',
					searchable: true,
					searchPlaceholder: 'Search versions...',
					submenuClass: 'w-[360px]',
					options: largeVersionOptions,
				},
			]
			function getVersionProject(categoryKey: string, optionValue: string) {
				if (categoryKey !== 'version') {
					return undefined
				}
				const optionIndex = Number(optionValue.replace('version-', '')) - 1
				return `Project ${Math.floor(optionIndex / 25) + 1}`
			}
			return { categories, getVersionProject, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories">
					<template #option-right="{ category, option }">
						<span
							v-if="getVersionProject(category.key, option.value)"
							v-tooltip="getVersionProject(category.key, option.value)"
							class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
						>
							<BoxIcon class="size-6" />
						</span>
					</template>
				</DropdownFilterBar>
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: [
			{
				key: 'version',
				label: 'Version',
				searchable: true,
				searchPlaceholder: 'Search versions...',
				submenuClass: 'w-[360px]',
				options: largeVersionOptions,
			},
		],
	},
	parameters: {
		docs: {
			description: {
				story:
					'Covers the add-menu submenu with flush rows, square hover states, and OverlayScrollbars.',
			},
		},
	},
}

export const CustomControls: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({
				version: ['1.21.5'],
			})
			const minimumDownloads = ref('1k')
			const releaseOnly = ref(true)
			const categories = [
				{
					key: 'version',
					label: 'Version',
					searchable: true,
					searchPlaceholder: 'Search versions...',
					submenuClass: 'w-[360px]',
					options: [
						{ value: '1.21.5', label: '1.21.5' },
						{ value: '1.21.4', label: '1.21.4' },
						{ value: '1.20.1', label: '1.20.1' },
						{ value: '25w15a', label: '25w15a' },
					],
				},
			]
			return { categories, minimumDownloads, releaseOnly, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories">
					<template #search-actions>
						<label class="ml-3 flex h-10 items-center gap-2 text-sm font-semibold text-secondary">
							<input v-model="releaseOnly" type="checkbox" />
							Release
						</label>
					</template>
					<template #category-footer="{ setSelectedValues }">
						<div class="border-0 border-t border-solid border-surface-5 px-6 py-2.5">
							<button
								type="button"
								class="border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
								@click="setSelectedValues(['1.21.5', '1.21.4'])"
							>
								Select recent versions
							</button>
						</div>
					</template>
					<template #preview-footer="{ category, setSelectedValues, closeMenu }">
						<div
							v-if="category.key === 'version'"
							class="flex flex-wrap items-center gap-3 border-0 border-t border-solid border-surface-5 px-6 py-2.5"
						>
							<span class="shrink-0 whitespace-nowrap text-sm font-semibold text-primary">
								Versions above
							</span>
							<input
								v-model="minimumDownloads"
								type="text"
								inputmode="numeric"
								class="h-8 w-16 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-center text-sm font-semibold text-primary outline-none"
								aria-label="Version downloads threshold"
								@keydown.enter.prevent.stop="setSelectedValues(['1.21.5', '1.21.4']); closeMenu($event)"
							/>
							<span class="shrink-0 text-sm font-semibold text-primary">downloads</span>
						</div>
					</template>
				</DropdownFilterBar>
			</div>
		`,
	}),
	args: {
		modelValue: {
			version: ['1.21.5'],
		},
		categories: searchableCategories,
	},
}

export const EmptyCategory: Story = {
	render: () => ({
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			const categories = [
				{
					key: 'empty',
					label: 'Empty',
					searchable: true,
					searchPlaceholder: 'Search empty options...',
					options: [],
				},
			]
			return { categories, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories" />
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: [
			{
				key: 'empty',
				label: 'Empty',
				searchable: true,
				searchPlaceholder: 'Search empty options...',
				options: [],
			},
		],
	},
}
