import type { Meta, StoryObj } from '@storybook/vue3-vite'

import LoadingIndicator from '../../components/base/LoadingIndicator.vue'

const meta = {
	title: 'Base/LoadingIndicator',
	component: LoadingIndicator,
} satisfies Meta<typeof LoadingIndicator>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
