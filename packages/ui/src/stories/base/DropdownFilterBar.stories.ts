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
			{ value: '1.21.5', label: '1.21.5' },
			{ value: '1.21.4', label: '1.21.4' },
			{ value: '1.20.1', label: '1.20.1' },
			{ value: '1.19.2', label: '1.19.2' },
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
			return { categories: defaultCategories, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories" />
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
		components: { DropdownFilterBar },
		setup() {
			const selected = ref<Record<string, string[]>>({})
			return { categories: searchableCategories, selected }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-center gap-2">
				<DropdownFilterBar v-model="selected" :categories="categories" />
			</div>
		`,
	}),
	args: {
		modelValue: {},
		categories: searchableCategories,
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
