import {
	DownloadIcon,
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
		triggerText: 'Select a loader',
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

export const IconSlotSearchable: Story = {
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
