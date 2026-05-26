import {
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	HeartIcon,
	SettingsIcon,
	ShareIcon,
	TrashIcon,
	UserIcon,
} from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Combobox from '../../components/base/Combobox.vue'

const meta = {
	title: 'Base/Combobox',
	// @ts-ignore - generic component
	component: Combobox,
} satisfies Meta<typeof Combobox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		options: [
			{ value: '1', label: 'Option 1' },
			{ value: '2', label: 'Option 2' },
			{ value: '3', label: 'Option 3' },
		],
		triggerText: 'Select an option',
	},
}

export const WithSelectedOption: Story = {
	args: {
		modelValue: '2',
		options: [
			{ value: '1', label: 'Option 1' },
			{ value: '2', label: 'Option 2' },
			{ value: '3', label: 'Option 3' },
		],
	},
}

export const Searchable: Story = {
	args: {
		options: [
			{ value: '1', label: 'Minecraft' },
			{ value: '2', label: 'Fabric' },
			{ value: '3', label: 'Forge' },
			{ value: '4', label: 'NeoForge' },
			{ value: '5', label: 'Quilt' },
		],
		searchable: true,
		searchPlaceholder: 'Search loaders...',
		selectSearchTextOnFocus: true,
	},
}

export const SearchableEmpty: Story = {
	args: {
		options: [],
		searchable: true,
		searchPlaceholder: 'Search projects...',
		noOptionsMessage: 'No projects found',
	},
	parameters: {
		docs: {
			description: {
				story:
					'Covers the idle empty searchable state: focusing the input should not open an empty dropdown until there is a query or footer content.',
			},
		},
	},
}

export const DropdownMinWidth: StoryObj = {
	render: () => ({
		components: { Combobox },
		data: () => ({
			selected: undefined,
			options: [
				{ value: 'fabric', label: 'Fabric', subLabel: 'Lightweight modding toolchain' },
				{ value: 'forge', label: 'Forge', subLabel: 'The original Minecraft modding API' },
				{ value: 'neoforge', label: 'NeoForge', subLabel: 'Community-driven Forge fork' },
			],
		}),
		template: /*html*/ `
			<div style="width: 11rem;">
				<Combobox
					v-model="selected"
					:options="options"
					:dropdown-min-width="320"
					placeholder="Loader"
				/>
			</div>
		`,
	}),
}

export const DropdownClass: StoryObj = {
	render: () => ({
		components: { Combobox },
		data: () => ({
			selected: undefined,
			options: [
				{ value: 'fabric', label: 'Fabric' },
				{ value: 'forge', label: 'Forge' },
				{ value: 'neoforge', label: 'NeoForge' },
			],
		}),
		template: /*html*/ `
			<div style="width: 14rem;">
				<Combobox
					v-model="selected"
					:options="options"
					dropdown-class="!border-brand"
					placeholder="Loader"
				/>
			</div>
		`,
	}),
}

export const Disabled: Story = {
	args: {
		options: [{ value: '1', label: 'Option 1' }],
		triggerText: 'Disabled',
		disabled: true,
	},
}

export const SearchableWithIcons: Story = {
	args: {
		options: [
			{ value: 'download', label: 'Download', icon: DownloadIcon },
			{ value: 'share', label: 'Share', icon: ShareIcon },
			{ value: 'favorite', label: 'Add to favorites', icon: HeartIcon },
			{ value: 'settings', label: 'Settings', icon: SettingsIcon },
			{ value: 'profile', label: 'Profile', icon: UserIcon },
			{ value: 'delete', label: 'Delete', icon: TrashIcon },
		],
		placeholder: 'Select an action',
		searchable: true,
		searchPlaceholder: 'Search actions...',
	},
}

export const WithDividers: Story = {
	args: {
		options: [
			{ value: 'download', label: 'Download', icon: DownloadIcon },
			{ value: 'share', label: 'Share', icon: ShareIcon },
			{ value: 'favorite', label: 'Add to favorites', icon: HeartIcon },
			{ type: 'divider' },
			{ value: 'settings', label: 'Settings', icon: SettingsIcon },
			{ value: 'profile', label: 'Profile', icon: UserIcon },
			{ type: 'divider' },
			{ value: 'delete', label: 'Delete', icon: TrashIcon, disabled: true },
		],
		placeholder: 'Select an action',
		listbox: false,
	},
}

export const WithSubLabel: Story = {
	args: {
		modelValue: '2',
		options: [
			{ value: '1', label: 'Fabric', subLabel: 'Lightweight modding toolchain' },
			{ value: '2', label: 'Forge', subLabel: 'The original Minecraft modding API' },
			{ value: '3', label: 'NeoForge', subLabel: 'Community-driven Forge fork' },
			{ value: '4', label: 'Quilt', subLabel: 'The mod-loader that cares' },
		],
	},
}

export const MixedSubLabels: Story = {
	args: {
		options: [
			{ value: '1', label: 'Minecraft', subLabel: 'The base game' },
			{ value: '2', label: 'Fabric' },
			{ value: '3', label: 'Forge', subLabel: 'Supports most mods' },
			{ value: '4', label: 'NeoForge' },
			{ value: '5', label: 'Quilt', subLabel: 'Fabric-compatible' },
		],
	},
}

export const SearchableNoFilter: Story = {
	args: {
		options: [
			{ value: 'download', label: 'Download', icon: DownloadIcon },
			{ value: 'share', label: 'Share', icon: ShareIcon },
			{ value: 'favorite', label: 'Add to favorites', icon: HeartIcon },
			{ value: 'settings', label: 'Settings', icon: SettingsIcon },
			{ value: 'profile', label: 'Profile', icon: UserIcon },
		],
		searchable: true,
		searchPlaceholder: 'Search actions...',
		disableSearchFilter: true,
	},
}

export const WithDropdownFooter: StoryObj = {
	render: () => ({
		components: { Combobox, EyeIcon, EyeOffIcon },
		data: () => ({
			selected: '1.20.4',
			showAll: false,
		}),
		computed: {
			options() {
				const releases = [
					{ value: '1.20.4', label: '1.20.4' },
					{ value: '1.20.3', label: '1.20.3' },
					{ value: '1.20.2', label: '1.20.2' },
					{ value: '1.20.1', label: '1.20.1' },
					{ value: '1.20', label: '1.20' },
				]
				const snapshots = [
					{ value: '24w03a', label: '24w03a' },
					{ value: '23w51b', label: '23w51b' },
				]
				// @ts-ignore
				return this.showAll ? [...releases, ...snapshots] : releases
			},
		},
		template: /*html*/ `
			<Combobox
				v-model="selected"
				:options="options"
				searchable
				placeholder="Select game version"
				search-placeholder="Search game versions"
			>
				<template #dropdown-footer>
					<button
						style="width: 100%; cursor: pointer; border: none; border-top: 1px solid var(--color-surface-5); background: transparent; padding: 0.75rem; text-align: center; font-size: 0.875rem; font-weight: 600; color: var(--color-secondary); transition: color 0.15s; display: flex; align-items: center; justify-content: center; gap: 0.375rem;"
						@mousedown.prevent
						@click="showAll = !showAll"
					>
						<EyeOffIcon v-if="showAll" style="width: 1rem; height: 1rem;" />
						<EyeIcon v-else style="width: 1rem; height: 1rem;" />
						{{ showAll ? 'Hide snapshots' : 'Show all versions' }}
					</button>
				</template>
			</Combobox>
		`,
	}),
}

export const DropdownFooterOnly: StoryObj = {
	render: () => ({
		components: { Combobox },
		data: () => ({
			selected: undefined,
			options: [],
		}),
		template: /*html*/ `
			<div style="width: 240px;">
				<Combobox
					v-model="selected"
					:options="options"
					display-value="Custom range"
					dropdown-min-width="320"
				>
					<template #dropdown-footer>
						<div style="display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; color: var(--color-text-primary);">
							<div style="font-size: 0.875rem; font-weight: 700;">Dropdown footer content</div>
							<div style="font-size: 0.8125rem; color: var(--color-text-secondary);">
								This dropdown has no options and stays open because its footer slot is content.
							</div>
							<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem;">
								<button type="button" style="height: 2rem; border: 1px solid var(--color-surface-5); border-radius: 0.5rem; background: var(--color-surface-3); color: var(--color-text-primary); font-weight: 600;">
									Cancel
								</button>
								<button type="button" style="height: 2rem; border: 0; border-radius: 0.5rem; background: var(--color-brand); color: var(--color-bg); font-weight: 700;">
									Apply
								</button>
							</div>
						</div>
					</template>
				</Combobox>
			</div>
		`,
	}),
	parameters: {
		docs: {
			description: {
				story:
					'Covers dropdowns whose only rendered content is the footer slot, such as the analytics custom date range picker.',
			},
		},
	},
}

/** Custom `#option` + `#search-selection-affix` (idle search field overlay), with `sync-with-selection`. */
export const SearchableWithOptionAndSelectionAffix: StoryObj = {
	render: () => ({
		components: { Combobox },
		data: () => ({
			selected: '100',
		}),
		computed: {
			options(): { value: string; label: string; tag?: 'BETA' }[] {
				return [
					{ value: '100', label: 'Build 100' },
					{ value: '99', label: 'Build 99', tag: 'BETA' },
				]
			},
		},
		template: /* html */ `
			<Combobox
				v-model="selected"
				:options="options"
				searchable
				sync-with-selection
				placeholder="Select build"
				search-placeholder="Search builds..."
			>
				<template #option="{ item, isSelected }">
					<div class="flex w-full items-center justify-between gap-2">
						<span
							class="font-semibold leading-tight"
							:class="isSelected ? 'text-contrast' : 'text-primary'"
						>
							{{ item.label }}
						</span>
						<span
							v-if="item.tag === 'BETA'"
							class="shrink-0 rounded-full bg-bg-orange px-2 text-sm font-bold text-orange"
						>
							Beta
						</span>
					</div>
				</template>
				<template #search-selection-affix="{ option }">
					<span
						v-if="option && option.tag === 'BETA'"
						class="shrink-0 rounded-full bg-bg-orange px-2 text-sm font-bold text-orange"
					>
						Beta
					</span>
				</template>
			</Combobox>
		`,
	}),
}

export const ManyOptionsOverflow: Story = {
	args: {
		options: Array.from({ length: 40 }, (_, index) => ({
			value: `${index + 1}`,
			label: `Option ${index + 1}`,
		})),
		placeholder: 'Select an option',
		maxHeight: 380,
	},
	parameters: {
		docs: {
			description: {
				story:
					'Covers long option lists where the dropdown content should scroll within its max height.',
			},
		},
	},
}

export const ScrollRepositioning: StoryObj = {
	render: () => ({
		components: { Combobox },
		data: () => ({
			selected: undefined,
			options: Array.from({ length: 16 }, (_, index) => ({
				value: `loader-${index + 1}`,
				label: `Loader ${index + 1}`,
			})),
		}),
		template: /*html*/ `
			<div style="min-height: 150vh; padding-top: 45vh;">
				<div style="width: min(100%, 22rem);">
					<Combobox
						v-model="selected"
						:options="options"
						searchable
						placeholder="Select loader"
						search-placeholder="Search loaders..."
					/>
				</div>
			</div>
		`,
	}),
	parameters: {
		docs: {
			description: {
				story:
					'Covers fixed dropdown repositioning while the page scrolls with a searchable input open.',
			},
		},
	},
}
