import { DownloadIcon, EyeIcon, FolderOpenIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { onMounted, onUnmounted, ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ContentCardTable from '../../components/instances/ContentCardTable.vue'
import type { ContentCardTableItem } from '../../components/instances/types'

// ============================================
// Fixtures
// ============================================

const fixtures = {
	sodium: {
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
			type: 'user' as const,
		},
		enabled: true,
	},
	modMenu: {
		id: 'mOgUt4GM',
		project: {
			id: 'mOgUt4GM',
			slug: 'modmenu',
			title: 'Mod Menu',
			icon_url:
				'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
		},
		version: {
			id: 'QuU0ciaR',
			version_number: '16.0.0',
			file_name: 'modmenu-16.0.0.jar',
		},
		owner: { id: 'u2', name: 'Prospector', type: 'user' as const },
		enabled: true,
	},
	fabricApi: {
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
			type: 'organization' as const,
		},
		enabled: false,
	},
} satisfies Record<string, ContentCardTableItem>

const defaultItems: ContentCardTableItem[] = [fixtures.sodium, fixtures.modMenu, fixtures.fabricApi]

/** Generate n items for stress testing */
function generateItems(count: number): ContentCardTableItem[] {
	return Array.from({ length: count }, (_, i) => ({
		...fixtures.sodium,
		id: `item-${i}`,
		project: { ...fixtures.sodium.project, title: `Mod ${i + 1}` },
		version: { ...fixtures.sodium.version!, version_number: `1.0.${i}` },
		enabled: i % 3 !== 0,
	}))
}

// ============================================
// Meta
// ============================================

const meta = {
	title: 'Instances/ContentCardTable',
	component: ContentCardTable,
	parameters: { layout: 'padded' },
	args: {
		items: defaultItems,
		showSelection: false,
		virtualized: true,
		'onUpdate:enabled': fn(),
		onDelete: fn(),
		onUpdate: fn(),
	},
} satisfies Meta<typeof ContentCardTable>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// Core Stories
// ============================================

export const Default: Story = {}

export const WithSelection: Story = {
	args: { showSelection: true },
	render: (args) => ({
		components: { ContentCardTable },
		setup() {
			const selectedIds = ref<string[]>([])
			return { args, selectedIds }
		},
		template: `
			<div class="flex flex-col gap-4">
				<ContentCardTable v-bind="args" v-model:selected-ids="selectedIds" />
				<p class="text-sm text-secondary">
					Selected: <strong>{{ selectedIds.length }}</strong>
					<span v-if="selectedIds.length"> ({{ selectedIds.join(', ') }})</span>
				</p>
			</div>
		`,
	}),
}

export const Empty: Story = {
	args: { items: [] },
}

export const EmptyCustom: Story = {
	args: { items: [] },
	render: (args) => ({
		components: { ContentCardTable, ButtonStyled },
		setup: () => ({ args }),
		template: `
			<ContentCardTable v-bind="args">
				<template #empty>
					<div class="flex flex-col items-center gap-4 py-8">
						<span class="text-lg text-secondary">No mods installed</span>
						<ButtonStyled color="green"><button>Browse mods</button></ButtonStyled>
					</div>
				</template>
			</ContentCardTable>
		`,
	}),
}

// ============================================
// States
// ============================================

/** All possible item states in one view */
export const AllStates: Story = {
	args: {
		showSelection: true,
		items: [
			{ ...fixtures.sodium, enabled: true },
			{ ...fixtures.modMenu, hasUpdate: true },
			{ ...fixtures.fabricApi, enabled: false },
			{
				id: 'long-name',
				project: {
					id: 'long-name',
					slug: 'long-mod',
					title: '[EMF] Entity Model Features - The Ultimate Entity Rendering Mod',
					icon_url: fixtures.sodium.project.icon_url,
				},
				version: {
					id: 'v1',
					version_number: '2.4.1-beta.15+mc1.21.1-fabric-loader0.16.0',
					file_name: 'emf-2.4.1-beta.15.jar',
				},
				owner: { id: 'u1', name: 'Traben', type: 'user' },
				enabled: true,
			},
			{
				id: 'no-icon',
				project: { id: 'no-icon', slug: 'imported', title: 'Imported mod', icon_url: undefined },
				version: { id: 'v1', version_number: 'Unknown', file_name: 'imported.jar' },
				enabled: true,
			},
			{
				id: 'no-avatar',
				project: {
					id: 'no-avatar',
					slug: 'no-avatar',
					title: 'No Owner Avatar',
					icon_url: fixtures.modMenu.project.icon_url,
				},
				version: { id: 'v1', version_number: '1.0.0', file_name: 'mod.jar' },
				owner: { id: 'u1', name: 'Anonymous', avatar_url: undefined, type: 'user' },
				enabled: true,
			},
			{ ...fixtures.modMenu, id: 'disabled-item', disabled: true, enabled: false },
		],
	},
	parameters: {
		docs: {
			description: {
				story:
					'Demonstrates: enabled, update available, disabled toggle, long names (truncation), missing icon, missing avatar, fully disabled item.',
			},
		},
	},
}

/** Items with update badges */
export const WithUpdates: Story = {
	args: {
		items: [
			{ ...fixtures.sodium, hasUpdate: true },
			{ ...fixtures.modMenu, hasUpdate: true },
			fixtures.fabricApi,
		],
	},
}

/** Per-item disabled state (e.g., during async operations) */
export const ItemsDisabled: Story = {
	args: {
		showSelection: true,
		items: [
			fixtures.sodium,
			{ ...fixtures.modMenu, disabled: true },
			{ ...fixtures.fabricApi, disabled: true },
		],
	},
	parameters: {
		docs: {
			description: { story: 'Items with `disabled: true` have all interactions disabled.' },
		},
	},
}

// ============================================
// Slots
// ============================================

export const CustomButtons: Story = {
	args: { showSelection: true },
	render: (args) => ({
		components: { ContentCardTable, ButtonStyled, EyeIcon, FolderOpenIcon, DownloadIcon },
		setup: () => ({ args }),
		template: `
			<ContentCardTable v-bind="args">
				<template #itemButtonsLeft="{ item }">
					<ButtonStyled v-tooltip="'Download'" circular type="transparent" color="green" color-fill="text">
						<button><DownloadIcon class="size-5" /></button>
					</ButtonStyled>
				</template>
				<template #itemButtonsRight="{ item }">
					<ButtonStyled v-tooltip="'View'" circular type="transparent">
						<button><EyeIcon class="size-5 text-secondary" /></button>
					</ButtonStyled>
					<ButtonStyled v-tooltip="'Folder'" circular type="transparent">
						<button><FolderOpenIcon class="size-5 text-secondary" /></button>
					</ButtonStyled>
				</template>
			</ContentCardTable>
		`,
	}),
}

export const WithOverflowMenu: Story = {
	args: {
		showSelection: true,
		items: [
			{
				...fixtures.sodium,
				overflowOptions: [
					{ id: 'view', action: () => console.log('View') },
					{ id: 'folder', action: () => console.log('Folder') },
					{ divider: true },
					{ id: 'remove', action: () => console.log('Remove'), color: 'red' as const },
				],
			},
			{
				...fixtures.modMenu,
				overflowOptions: [
					{ id: 'view', action: () => console.log('View') },
					{ divider: true },
					{ id: 'remove', action: () => console.log('Remove'), color: 'red' as const },
				],
			},
		],
	},
	render: (args) => ({
		components: { ContentCardTable },
		setup: () => ({ args }),
		template: `
			<ContentCardTable v-bind="args">
				<template #view>View on Modrinth</template>
				<template #folder>Open folder</template>
				<template #remove>Remove</template>
			</ContentCardTable>
		`,
	}),
}

// ============================================
// Interactive
// ============================================

export const Interactive: Story = {
	args: { showSelection: true },
	render: (args) => ({
		components: { ContentCardTable },
		setup() {
			const items = ref<ContentCardTableItem[]>(
				defaultItems.map((item) => ({ ...item, enabled: item.id !== fixtures.fabricApi.id })),
			)
			const selectedIds = ref<string[]>([])

			const handleToggle = (id: string, value: boolean) => {
				const item = items.value.find((i) => i.id === id)
				if (item) item.enabled = value
			}

			const handleDelete = (id: string) => {
				items.value = items.value.filter((i) => i.id !== id)
				selectedIds.value = selectedIds.value.filter((i) => i !== id)
			}

			return { args, items, selectedIds, handleToggle, handleDelete }
		},
		template: `
			<div class="flex flex-col gap-4">
				<ContentCardTable
					:items="items"
					:show-selection="args.showSelection"
					v-model:selected-ids="selectedIds"
					@update:enabled="handleToggle"
					@delete="handleDelete"
				/>
				<p class="text-sm text-secondary">
					Items: <strong>{{ items.length }}</strong> · Selected: <strong>{{ selectedIds.length }}</strong>
				</p>
			</div>
		`,
	}),
}

export const BulkActions: Story = {
	render: () => ({
		components: { ContentCardTable, ButtonStyled },
		setup() {
			const items = ref<ContentCardTableItem[]>(
				defaultItems.map((item, i) => ({ ...item, enabled: i !== 2 })),
			)
			const selectedIds = ref<string[]>([])

			const setEnabled = (value: boolean) => {
				items.value.forEach((item) => {
					if (selectedIds.value.includes(item.id)) item.enabled = value
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

			return { items, selectedIds, setEnabled, deleteSelected, handleToggle }
		},
		template: `
			<div class="flex flex-col gap-4">
				<div class="flex items-center gap-2">
					<span class="text-sm text-secondary">{{ selectedIds.length }} selected</span>
					<template v-if="selectedIds.length">
						<ButtonStyled size="small" color="green">
							<button @click="setEnabled(true)">Enable</button>
						</ButtonStyled>
						<ButtonStyled size="small" type="transparent">
							<button @click="setEnabled(false)">Disable</button>
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
				/>
			</div>
		`,
	}),
}

// ============================================
// Performance
// ============================================

export const Virtualization: Story = {
	parameters: {
		docs: {
			description: {
				story:
					'2000 items with virtualization. Toggle to compare DOM node count. Virtualized should render ~20-30 nodes vs 2000.',
			},
		},
	},
	render: () => ({
		components: { ContentCardTable },
		setup() {
			const items = ref(generateItems(2000))
			const selectedIds = ref<string[]>([])
			const virtualized = ref(true)
			const tableRef = ref<InstanceType<typeof ContentCardTable> | null>(null)
			const domNodes = ref(0)
			let raf: number

			const updateNodeCount = () => {
				if (tableRef.value?.$el) {
					domNodes.value = (tableRef.value.$el as HTMLElement).querySelectorAll(
						'[data-content-card-item]',
					).length
				}
				raf = requestAnimationFrame(updateNodeCount)
			}

			onMounted(() => {
				raf = requestAnimationFrame(updateNodeCount)
			})
			onUnmounted(() => cancelAnimationFrame(raf))

			return { items, selectedIds, virtualized, tableRef, domNodes }
		},
		template: `
			<div>
				<div class="sticky top-0 z-10 mb-4 flex items-center gap-3 rounded-lg bg-surface-2 p-3">
					<label class="flex cursor-pointer items-center gap-2">
						<input type="checkbox" v-model="virtualized" class="h-4 w-4 rounded" />
						<span class="font-medium text-contrast">Virtualization</span>
					</label>
					<span class="ml-auto font-mono text-sm">
						DOM: <span :class="domNodes > 100 ? 'text-red-500' : 'text-green-500'">{{ domNodes }}</span>
						/ {{ items.length }}
					</span>
				</div>
				<ContentCardTable
					ref="tableRef"
					:items="items"
					:virtualized="virtualized"
					show-selection
					v-model:selected-ids="selectedIds"
				/>
			</div>
		`,
	}),
}
