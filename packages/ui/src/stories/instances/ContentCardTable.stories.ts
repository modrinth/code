import { DownloadIcon, EyeIcon, FolderOpenIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ContentCardTable from '../../components/instances/ContentCardTable.vue'
import type { ContentCardTableItem } from '../../components/instances/types'

// Sample data
const sodiumItem: ContentCardTableItem = {
	id: 'AANobbMI',
	project: {
		id: 'AANobbMI',
		slug: 'sodium',
		title: 'Sodium',
		icon_url:
			'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
	},
	version: {
		id: '59wygFUQ',
		version_number: 'mc1.21.11-0.8.2-fabric',
		file_name: 'sodium-fabric-0.8.2+mc1.21.11.jar',
	},
	owner: {
		id: 'DzLrfrbK',
		name: 'IMS',
		avatar_url: 'https://avatars3.githubusercontent.com/u/31803019?v=4',
		type: 'user',
	},
	enabled: true,
}

const modMenuItem: ContentCardTableItem = {
	id: 'mOgUt4GM',
	project: {
		id: 'mOgUt4GM',
		slug: 'modmenu',
		title: 'Mod Menu',
		icon_url: 'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
	},
	version: {
		id: 'QuU0ciaR',
		version_number: '16.0.0',
		file_name: 'modmenu-16.0.0.jar',
	},
	owner: {
		id: 'u2',
		name: 'Prospector',
		type: 'user',
	},
	enabled: true,
}

const fabricApiItem: ContentCardTableItem = {
	id: 'P7dR8mSH',
	project: {
		id: 'P7dR8mSH',
		slug: 'fabric-api',
		title: 'Fabric API',
		icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
	},
	version: {
		id: 'Lwa1Q6e4',
		version_number: '0.141.3+26.1',
		file_name: 'fabric-api-0.141.3+26.1.jar',
	},
	owner: {
		id: 'BZoBsPo6',
		name: 'FabricMC',
		avatar_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
		type: 'organization',
	},
	enabled: false,
}

const emfItem: ContentCardTableItem = {
	id: 'emf123',
	project: {
		id: 'emf123',
		slug: 'entity-model-features',
		title: '[EMF] Entity Model Features',
		icon_url:
			'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
	},
	version: {
		id: 'v1',
		version_number: '2.4.1',
		file_name: 'Entity_model_features_fabric_1.21.1-2.4.1.jar',
	},
	owner: {
		id: 'u1',
		name: 'Traben',
		type: 'user',
	},
	enabled: true,
}

const etfItem: ContentCardTableItem = {
	id: 'etf456',
	project: {
		id: 'etf456',
		slug: 'entity-texture-features',
		title: '[ETF] Entity Texture Features',
		icon_url: 'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
	},
	version: {
		id: 'v2',
		version_number: '6.2.9',
		file_name: 'Entity_texture_features_fabric_1.21.1-6.2.9.jar',
	},
	owner: {
		id: 'u1',
		name: 'Traben',
		type: 'user',
	},
	enabled: true,
}

const importedModItem: ContentCardTableItem = {
	id: 'imported123',
	project: {
		id: 'imported123',
		slug: 'import-mod',
		title: 'Import mod',
		icon_url: undefined,
	},
	version: {
		id: 'v3',
		version_number: 'Unknown',
		file_name: 'Entity_texture_features_fabric_1.21.1-6.2.9.jar',
	},
	enabled: false,
}

const sampleItems: ContentCardTableItem[] = [sodiumItem, modMenuItem, fabricApiItem]

const figmaDesignItems: ContentCardTableItem[] = [emfItem, etfItem, importedModItem]

const meta = {
	title: 'Instances/ContentCardTable',
	component: ContentCardTable,
	parameters: {
		layout: 'padded',
	},
	argTypes: {
		items: {
			control: 'object',
			description: 'Array of items to display in the table',
		},
		showSelection: {
			control: 'boolean',
			description: 'Show checkboxes for selection',
		},
		sortable: {
			control: 'boolean',
			description: 'Enable column sorting',
		},
		sortBy: {
			control: 'select',
			options: ['project', 'version', undefined],
			description: 'Current sort column',
		},
		sortDirection: {
			control: 'select',
			options: ['asc', 'desc'],
			description: 'Sort direction',
		},
	},
} satisfies Meta<typeof ContentCardTable>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// Basic Stories
// ============================================

export const Default: Story = {
	args: {
		items: sampleItems,
	},
}

export const FigmaDesign: Story = {
	args: {
		items: figmaDesignItems,
		showSelection: true,
	},
	render: (args) => ({
		components: { ContentCardTable },
		setup() {
			const selectedIds = ref<string[]>([emfItem.id, etfItem.id])
			return { args, selectedIds }
		},
		template: /*html*/ `
			<ContentCardTable
				v-bind="args"
				v-model:selected-ids="selectedIds"
				@update:enabled="(id, val) => console.log('Toggle', id, val)"
				@delete="(id) => console.log('Delete', id)"
			/>
		`,
	}),
}

export const WithSelection: Story = {
	args: {
		items: sampleItems,
		showSelection: true,
	},
	render: (args) => ({
		components: { ContentCardTable },
		setup() {
			const selectedIds = ref<string[]>([])
			return { args, selectedIds }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<ContentCardTable
					v-bind="args"
					v-model:selected-ids="selectedIds"
					@update:enabled="(id, val) => console.log('Toggle', id, val)"
					@delete="(id) => console.log('Delete', id)"
				/>
				<div class="text-sm text-secondary">
					Selected: <strong>{{ selectedIds.length }}</strong> items
					<span v-if="selectedIds.length">({{ selectedIds.join(', ') }})</span>
				</div>
			</div>
		`,
	}),
}

export const WithSorting: Story = {
	args: {
		items: sampleItems,
		sortable: true,
		sortBy: 'project',
		sortDirection: 'asc',
	},
	render: (args) => ({
		components: { ContentCardTable },
		setup() {
			const sortBy = ref<'project' | 'version' | undefined>(args.sortBy)
			const sortDirection = ref<'asc' | 'desc'>(args.sortDirection || 'asc')

			const handleSort = (column: 'project' | 'version', direction: 'asc' | 'desc') => {
				sortBy.value = column
				sortDirection.value = direction
				console.log('Sort:', column, direction)
			}

			return { args, sortBy, sortDirection, handleSort }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<ContentCardTable
					:items="args.items"
					:sortable="args.sortable"
					:sort-by="sortBy"
					:sort-direction="sortDirection"
					@sort="handleSort"
				/>
				<div class="text-sm text-secondary">
					Sorted by: <strong>{{ sortBy || 'none' }}</strong> ({{ sortDirection }})
				</div>
			</div>
		`,
	}),
}

export const WithSelectionAndSorting: Story = {
	args: {
		items: sampleItems,
		showSelection: true,
		sortable: true,
		sortBy: 'project',
		sortDirection: 'asc',
	},
	render: (args) => ({
		components: { ContentCardTable },
		setup() {
			const selectedIds = ref<string[]>([])
			const sortBy = ref<'project' | 'version' | undefined>(args.sortBy)
			const sortDirection = ref<'asc' | 'desc'>(args.sortDirection || 'asc')

			const handleSort = (column: 'project' | 'version', direction: 'asc' | 'desc') => {
				sortBy.value = column
				sortDirection.value = direction
			}

			return { args, selectedIds, sortBy, sortDirection, handleSort }
		},
		template: /*html*/ `
			<ContentCardTable
				:items="args.items"
				:show-selection="args.showSelection"
				:sortable="args.sortable"
				:sort-by="sortBy"
				:sort-direction="sortDirection"
				v-model:selected-ids="selectedIds"
				@sort="handleSort"
				@update:enabled="(id, val) => console.log('Toggle', id, val)"
				@delete="(id) => console.log('Delete', id)"
			/>
		`,
	}),
}

// ============================================
// Action Stories
// ============================================

export const WithActions: Story = {
	args: {
		items: sampleItems,
		showSelection: true,
		'onUpdate:enabled': fn(),
		onDelete: fn(),
		onUpdate: fn(),
	},
}

export const InteractiveActions: Story = {
	render: () => ({
		components: { ContentCardTable },
		setup() {
			const items = ref<ContentCardTableItem[]>([
				{ ...sodiumItem, enabled: true },
				{ ...modMenuItem, enabled: true },
				{ ...fabricApiItem, enabled: false },
			])
			const selectedIds = ref<string[]>([])

			const handleToggle = (id: string, value: boolean) => {
				const item = items.value.find((i) => i.id === id)
				if (item) item.enabled = value
			}

			const handleDelete = (id: string) => {
				items.value = items.value.filter((i) => i.id !== id)
				selectedIds.value = selectedIds.value.filter((i) => i !== id)
			}

			const handleUpdate = (id: string) => {
				console.log('Update available clicked for:', id)
			}

			return { items, selectedIds, handleToggle, handleDelete, handleUpdate }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<ContentCardTable
					:items="items"
					show-selection
					v-model:selected-ids="selectedIds"
					@update:enabled="handleToggle"
					@delete="handleDelete"
					@update="handleUpdate"
				/>
				<div class="flex gap-4 text-sm text-secondary">
					<span>Items: <strong>{{ items.length }}</strong></span>
					<span>Selected: <strong>{{ selectedIds.length }}</strong></span>
				</div>
			</div>
		`,
	}),
}

// ============================================
// Slot Stories
// ============================================

export const WithCustomItemButtons: Story = {
	render: () => ({
		components: { ContentCardTable, ButtonStyled, EyeIcon, FolderOpenIcon, DownloadIcon },
		setup() {
			return { items: sampleItems }
		},
		template: /*html*/ `
			<ContentCardTable
				:items="items"
				show-selection
				@update:enabled="(id, val) => console.log('Toggle', id, val)"
				@delete="(id) => console.log('Delete', id)"
			>
				<template #itemButtonsLeft="{ item }">
					<ButtonStyled v-tooltip="'Download'" circular type="transparent" color="green" color-fill="text">
						<button @click="console.log('Download', item.id)">
							<DownloadIcon class="size-5" />
						</button>
					</ButtonStyled>
				</template>
				<template #itemButtonsRight="{ item }">
					<ButtonStyled v-tooltip="'View on Modrinth'" circular type="transparent">
						<button @click="console.log('View', item.id)">
							<EyeIcon class="size-5 text-secondary" />
						</button>
					</ButtonStyled>
					<ButtonStyled v-tooltip="'Open folder'" circular type="transparent">
						<button @click="console.log('Open folder', item.id)">
							<FolderOpenIcon class="size-5 text-secondary" />
						</button>
					</ButtonStyled>
				</template>
			</ContentCardTable>
		`,
	}),
}

export const WithEmptyState: Story = {
	args: {
		items: [],
	},
}

export const WithCustomEmptyState: Story = {
	render: () => ({
		components: { ContentCardTable, ButtonStyled },
		template: /*html*/ `
			<ContentCardTable :items="[]">
				<template #empty>
					<div class="flex flex-col items-center gap-4 py-8">
						<span class="text-lg text-secondary">No mods installed</span>
						<ButtonStyled color="green">
							<button>Browse mods</button>
						</ButtonStyled>
					</div>
				</template>
			</ContentCardTable>
		`,
	}),
}

// ============================================
// State Stories
// ============================================

export const PerItemDisabled: Story = {
	render: () => ({
		components: { ContentCardTable },
		setup() {
			// Simulates items being modified (e.g., toggled, deleted)
			const items: ContentCardTableItem[] = [
				{ ...sodiumItem, enabled: true },
				{ ...modMenuItem, enabled: true, disabled: true }, // Being modified
				{ ...fabricApiItem, enabled: false, disabled: true }, // Being modified
			]
			return { items }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<p class="text-sm text-secondary">
					Items with <code>disabled: true</code> have all interactions disabled (simulating items being modified).
				</p>
				<ContentCardTable
					:items="items"
					show-selection
					@update:enabled="(id, val) => console.log('Toggle', id, val)"
					@delete="(id) => console.log('Delete', id)"
				/>
			</div>
		`,
	}),
}

export const SingleItem: Story = {
	args: {
		items: [sodiumItem],
		showSelection: true,
	},
}

export const ManyItems: Story = {
	render: () => ({
		components: { ContentCardTable },
		setup() {
			const items = ref<ContentCardTableItem[]>(
				Array.from({ length: 20 }, (_, i) => ({
					...sodiumItem,
					id: `item-${i}`,
					project: {
						...sodiumItem.project,
						title: `Mod ${i + 1}`,
					},
					version: {
						...sodiumItem.version!,
						version_number: `1.0.${i}`,
					},
					enabled: i % 3 !== 0,
				})),
			)
			const selectedIds = ref<string[]>([])

			return { items, selectedIds }
		},
		template: /*html*/ `
			<div class="max-h-[600px] overflow-auto">
				<ContentCardTable
					:items="items"
					show-selection
					v-model:selected-ids="selectedIds"
					@update:enabled="(id, val) => console.log('Toggle', id, val)"
					@delete="(id) => console.log('Delete', id)"
				/>
			</div>
		`,
	}),
}

// ============================================
// With Overflow Menu
// ============================================

export const WithOverflowMenu: Story = {
	render: () => ({
		components: { ContentCardTable },
		setup() {
			const items: ContentCardTableItem[] = [
				{
					...sodiumItem,
					overflowOptions: [
						{ id: 'view', action: () => console.log('View sodium') },
						{ id: 'folder', action: () => console.log('Open folder') },
						{ divider: true },
						{ id: 'remove', action: () => console.log('Remove'), color: 'red' as const },
					],
				},
				{
					...modMenuItem,
					overflowOptions: [
						{ id: 'view', action: () => console.log('View modmenu') },
						{ divider: true },
						{ id: 'remove', action: () => console.log('Remove'), color: 'red' as const },
					],
				},
			]

			return { items }
		},
		template: /*html*/ `
			<ContentCardTable
				:items="items"
				show-selection
				@update:enabled="(id, val) => console.log('Toggle', id, val)"
				@delete="(id) => console.log('Delete', id)"
			>
				<template #view>View on Modrinth</template>
				<template #folder>Open folder</template>
				<template #remove>Remove</template>
			</ContentCardTable>
		`,
	}),
}

// ============================================
// Bulk Actions Demo
// ============================================

export const BulkActionsDemo: Story = {
	render: () => ({
		components: { ContentCardTable, ButtonStyled },
		setup() {
			const items = ref<ContentCardTableItem[]>([
				{ ...sodiumItem, enabled: true },
				{ ...modMenuItem, enabled: true },
				{ ...fabricApiItem, enabled: false },
				{ ...emfItem, enabled: true },
				{ ...etfItem, enabled: true },
			])
			const selectedIds = ref<string[]>([])

			const enableSelected = () => {
				items.value.forEach((item) => {
					if (selectedIds.value.includes(item.id)) {
						item.enabled = true
					}
				})
			}

			const disableSelected = () => {
				items.value.forEach((item) => {
					if (selectedIds.value.includes(item.id)) {
						item.enabled = false
					}
				})
			}

			const deleteSelected = () => {
				items.value = items.value.filter((item) => !selectedIds.value.includes(item.id))
				selectedIds.value = []
			}

			const handleToggle = (id: string, value: boolean) => {
				const item = items.value.find((i) => i.id === id)
				if (item) item.enabled = value
			}

			return { items, selectedIds, enableSelected, disableSelected, deleteSelected, handleToggle }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<div class="flex items-center gap-2">
					<span class="text-sm text-secondary">{{ selectedIds.length }} selected</span>
					<template v-if="selectedIds.length > 0">
						<ButtonStyled size="small" color="green">
							<button @click="enableSelected">Enable</button>
						</ButtonStyled>
						<ButtonStyled size="small" type="transparent">
							<button @click="disableSelected">Disable</button>
						</ButtonStyled>
						<ButtonStyled size="small" color="red">
							<button @click="deleteSelected">Delete</button>
						</ButtonStyled>
					</template>
				</div>
				<ContentCardTable
					:items="items"
					show-selection
					v-model:selected-ids="selectedIds"
					@update:enabled="handleToggle"
					@delete="(id) => console.log('Delete', id)"
				/>
			</div>
		`,
	}),
}
