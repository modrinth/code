import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

import ContentTable from '../../layouts/shared/content-tab/components/content-table/index.vue'
import type { ContentTableColumns } from '../../layouts/shared/content-tab/components/content-table/types'
import type { ContentCardTableItem } from '../../layouts/shared/content-tab/types'

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
	external: true,
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

const longNameItem: ContentCardTableItem = {
	id: 'long-name',
	project: {
		id: 'long-name',
		slug: 'very-long-project-name',
		title: '[EMF] Entity Model Features - The Ultimate Entity Rendering Mod for Minecraft',
		icon_url: sodiumItem.project.icon_url,
	},
	version: {
		id: 'v1',
		version_number: '2.4.1-beta.15+mc1.21.1-fabric-loader0.16.0',
		file_name: 'Entity_model_features_fabric_1.21.1-2.4.1-beta.15+mc1.21.1-fabric-loader0.16.0.jar',
	},
	owner: {
		id: 'u1',
		name: 'Traben',
		type: 'user',
	},
	enabled: true,
}

const noOwnerAvatarItem: ContentCardTableItem = {
	id: 'no-avatar',
	project: {
		id: 'no-avatar',
		slug: 'no-avatar-mod',
		title: 'Mod Without Owner Avatar',
		icon_url: modMenuItem.project.icon_url,
	},
	version: {
		id: 'v1',
		version_number: '1.0.0',
		file_name: 'no-avatar-mod-1.0.0.jar',
	},
	owner: {
		id: 'u1',
		name: 'Anonymous User',
		avatar_url: undefined,
		type: 'user',
	},
	enabled: true,
}

const updateAvailableItem: ContentCardTableItem = {
	id: 'update-available',
	project: {
		id: 'update-available',
		slug: 'outdated-mod',
		title: 'Outdated Mod',
		icon_url: fabricApiItem.project.icon_url,
	},
	version: {
		id: 'v1',
		version_number: '1.0.0',
		file_name: 'outdated-mod-1.0.0.jar',
	},
	owner: fabricApiItem.owner,
	enabled: true,
	hasUpdate: true,
}

const sampleItems: ContentCardTableItem[] = [sodiumItem, modMenuItem, fabricApiItem]

const allStatesItems: ContentCardTableItem[] = [
	{ ...sodiumItem, enabled: true, hasUpdate: false },
	{ ...modMenuItem, enabled: true, hasUpdate: true },
	{ ...fabricApiItem, enabled: false },
	longNameItem,
	importedModItem,
	noOwnerAvatarItem,
	updateAvailableItem,
	{ ...emfItem, locked: true },
	{ ...etfItem, synced: true, syncUpdatePending: true },
	{ ...etfItem, id: 'disabled-row', disabled: true, disabledTooltip: 'Please wait' },
	{ ...sodiumItem, id: 'installing-row', installing: true, installProgress: 40 },
	{ ...modMenuItem, id: 'warning-row', clientWarning: 'environment' },
]

const instanceColumns: ContentTableColumns = {
	selection: true,
	actions: ['update', 'switchVersion', 'toggle', 'remove'],
}

const meta = {
	title: 'Instances/ContentTable',
	component: ContentTable,
	parameters: {
		layout: 'padded',
	},
	args: {
		sections: [{ id: 'content', items: sampleItems }],
		columns: instanceColumns,
	},
} satisfies Meta<typeof ContentTable>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const AllStates: Story = {
	render: () => ({
		components: { ContentTable },
		setup() {
			const selectedIds = ref<string[]>([])
			return { selectedIds, items: allStatesItems, columns: instanceColumns }
		},
		template: /*html*/ `
			<ContentTable
				v-model:selected-ids="selectedIds"
				:sections="[{ id: 'content', items }]"
				:columns="columns"
				@toggle="(id, value) => console.log('Toggle', id, value)"
				@remove="(id) => console.log('Remove', id)"
				@update="(id) => console.log('Update', id)"
				@switch-version="(id) => console.log('Switch version', id)"
			/>
		`,
	}),
}

export const ReadOnly: Story = {
	args: {
		sections: [{ id: 'content', items: [sodiumItem, longNameItem, importedModItem] }],
		columns: {},
	},
}

export const ActionsWithoutToggle: Story = {
	args: {
		sections: [{ id: 'content', items: allStatesItems }],
		columns: { actions: ['remove'] },
	},
}

export const EnabledForColumn: Story = {
	render: () => ({
		components: { ContentTable },
		setup() {
			const items = ref<ContentCardTableItem[]>(
				[sodiumItem, modMenuItem, fabricApiItem, emfItem].map((item, index) => ({
					...item,
					enabledFor: {
						server: index % 2 === 0,
						player: true,
						locked: index === 3,
						lockedTooltip:
							index === 3 ? 'Resource packs can only be enabled for players.' : undefined,
						disabledSides: index === 3 ? ['server'] : [],
					},
				})),
			)
			function setSide(id: string, side: 'server' | 'player', value: boolean) {
				items.value = items.value.map((item) =>
					item.id === id && item.enabledFor
						? { ...item, enabledFor: { ...item.enabledFor, [side]: value } }
						: item,
				)
			}
			return { items, setSide }
		},
		template: /*html*/ `
			<ContentTable
				:sections="[{ id: 'content', items }]"
				:columns="{ selection: true, enabledFor: true, actions: ['toggle', 'remove'] }"
				@enabled-for="setSide"
			/>
		`,
	}),
}

export const LabelledSections: Story = {
	args: {
		sections: [
			{ id: 'group-a', label: 'Group A - 1', items: [updateAvailableItem] },
			{ id: 'ungrouped', label: 'Ungrouped - 2', items: [sodiumItem, fabricApiItem] },
		],
		columns: instanceColumns,
	},
}

export const DraggableGroups: Story = {
	render: () => ({
		components: { ContentTable },
		setup() {
			const groups = ref([
				{ id: 'performance', label: 'Performance', items: [sodiumItem.id] },
				{ id: 'empty', label: 'Empty group', items: [] as string[] },
				{
					id: 'ungrouped',
					label: 'Ungrouped',
					items: [updateAvailableItem.id, fabricApiItem.id, modMenuItem.id],
				},
			])
			const collapsed = ref<Record<string, boolean>>({})
			const selectedIds = ref<string[]>([])
			const itemsById = new Map(
				[sodiumItem, updateAvailableItem, fabricApiItem, modMenuItem].map((item) => [
					item.id,
					item,
				]),
			)
			const sections = computed(() =>
				groups.value.map((group) => ({
					id: group.id,
					label: group.label,
					collapsed: !!collapsed.value[group.id],
					emptyLabel:
						group.id === 'ungrouped' ? undefined : 'Drag content here to add it to this group',
					items: group.items.map((id) => itemsById.get(id)!),
				})),
			)
			function setCollapsed(id: string, value: boolean) {
				collapsed.value = { ...collapsed.value, [id]: value }
			}
			function moveRows(ids: string[], sectionId: string) {
				groups.value = groups.value.map((group) => ({
					...group,
					items:
						group.id === sectionId
							? [...group.items.filter((id) => !ids.includes(id)), ...ids]
							: group.items.filter((id) => !ids.includes(id)),
				}))
				selectedIds.value = []
			}
			return { sections, selectedIds, setCollapsed, moveRows, columns: instanceColumns }
		},
		template: /*html*/ `
			<ContentTable
				v-model:selected-ids="selectedIds"
				:sections="sections"
				:columns="columns"
				draggable
				@set-section-collapsed="setCollapsed"
				@move-rows="moveRows"
			/>
		`,
	}),
}

export const Highlighted: Story = {
	args: {
		sections: [{ id: 'content', items: allStatesItems }],
		columns: instanceColumns,
		highlightedItemId: modMenuItem.id,
	},
}

export const Empty: Story = {
	render: () => ({
		components: { ContentTable },
		setup() {
			return { columns: instanceColumns }
		},
		template: /*html*/ `
			<ContentTable :sections="[{ id: 'content', items: [] }]" :columns="columns">
				<template #empty>
					<span class="text-secondary">No content found.</span>
				</template>
			</ContentTable>
		`,
	}),
}

export const ManyRows: Story = {
	args: {
		sections: [
			{
				id: 'content',
				items: Array.from({ length: 500 }, (_, index) => ({
					...sampleItems[index % sampleItems.length],
					id: `row-${index}`,
					project: {
						...sampleItems[index % sampleItems.length].project,
						title: `${sampleItems[index % sampleItems.length].project.title} ${index + 1}`,
					},
				})),
			},
		],
		columns: instanceColumns,
	},
}
