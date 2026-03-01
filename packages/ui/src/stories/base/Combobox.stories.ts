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
	},
}

export const Disabled: Story = {
	args: {
		options: [{ value: '1', label: 'Option 1' }],
		triggerText: 'Disabled',
		disabled: true,
	},
}

export const IconSlot: Story = {
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

export const WithSelectedOption: Story = {
	args: {
		options: [
			{ value: '1', label: 'Option 1' },
			{ value: '2', label: 'Option 2' },
			{ value: '3', label: 'Option 3' },
		],
		modelValue: '2',
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

export const SearchableModpacks: Story = {
	args: {
		options: [
			{ value: 'download', label: 'Download', icon: DownloadIcon },
			{ value: 'share', label: 'Share', icon: ShareIcon },
			{ value: 'favorite', label: 'Add to favorites', icon: HeartIcon },
			{ value: 'settings', label: 'Settings', icon: SettingsIcon },
		],
		searchable: true,
		searchPlaceholder: 'Search modpacks...',
		noOptionsMessage: 'No modpacks found',
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

export const WithSelectedOptionAndIcon: Story = {
	args: {
		options: [
			{ value: 'download', label: 'Download', icon: DownloadIcon },
			{ value: 'share', label: 'Share', icon: ShareIcon },
			{ value: 'favorite', label: 'Add to favorites', icon: HeartIcon },
			{ value: 'settings', label: 'Settings', icon: SettingsIcon },
			{ value: 'profile', label: 'Profile', icon: UserIcon },
		],
		modelValue: 'favorite',
		showIconInSelected: true,
	},
}
