import type { Meta, StoryObj } from '@storybook/vue3-vite'

import SettingsLabel from '../../components/base/SettingsLabel.vue'

const meta = {
	title: 'Base/SettingsLabel',
	component: SettingsLabel,
} satisfies Meta<typeof SettingsLabel>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		title: 'Setting Name',
	},
}

export const WithDescription: Story = {
	args: {
		title: 'Enable Notifications',
		description: 'Receive email notifications when someone follows your project.',
	},
}

export const WithId: Story = {
	args: {
		id: 'setting-input',
		title: 'Username',
		description: 'Your unique username on the platform.',
	},
}
