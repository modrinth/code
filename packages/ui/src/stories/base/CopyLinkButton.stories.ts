import type { Meta, StoryObj } from '@storybook/vue3-vite'

import CopyLinkButton from '../../components/base/CopyLinkButton.vue'

const meta = {
	title: 'Base/CopyLinkButton',
	component: CopyLinkButton,
} satisfies Meta<typeof CopyLinkButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		url: 'https://modrinth.com',
	},
}

export const CustomLabels: Story = {
	args: {
		url: 'https://modrinth.com',
		copyLabel: 'Copy project link',
		copiedLabel: 'Project link copied',
	},
}
