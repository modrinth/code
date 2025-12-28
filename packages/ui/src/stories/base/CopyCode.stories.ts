import type { Meta, StoryObj } from '@storybook/vue3-vite'

import CopyCode from '../../components/base/CopyCode.vue'

const meta = {
	title: 'Base/CopyCode',
	component: CopyCode,
} satisfies Meta<typeof CopyCode>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		text: 'npm install @modrinth/ui',
	},
}

export const LongText: Story = {
	args: {
		text: 'curl -X GET "https://api.modrinth.com/v2/project/sodium" -H "Accept: application/json"',
	},
}
