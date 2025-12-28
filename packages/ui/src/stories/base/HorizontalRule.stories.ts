import type { Meta, StoryObj } from '@storybook/vue3-vite'

import HorizontalRule from '../../components/base/HorizontalRule.vue'

const meta = {
	title: 'Base/HorizontalRule',
	component: HorizontalRule,
} satisfies Meta<typeof HorizontalRule>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
