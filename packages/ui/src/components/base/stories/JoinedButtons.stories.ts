import { DownloadIcon, HeartIcon, ShareIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import JoinedButtons from '../JoinedButtons.vue'

const meta = {
	title: 'Base/JoinedButtons',
	component: JoinedButtons,
} satisfies Meta<typeof JoinedButtons>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		actions: [
			{ id: 'download', label: 'Download', icon: DownloadIcon, action: () => {} },
			{ id: 'follow', label: 'Follow', icon: HeartIcon, action: () => {} },
			{ id: 'share', label: 'Share', icon: ShareIcon, action: () => {} },
		],
	},
}

export const Brand: Story = {
	args: {
		color: 'brand',
		actions: [
			{ id: 'download', label: 'Download', icon: DownloadIcon, action: () => {} },
			{ id: 'follow', label: 'Follow', icon: HeartIcon, action: () => {} },
		],
	},
}

export const SingleAction: Story = {
	args: {
		actions: [{ id: 'download', label: 'Download', icon: DownloadIcon, action: () => {} }],
	},
}

export const Disabled: Story = {
	args: {
		disabled: true,
		actions: [
			{ id: 'download', label: 'Download', icon: DownloadIcon, action: () => {} },
			{ id: 'follow', label: 'Follow', icon: HeartIcon, action: () => {} },
		],
	},
}
