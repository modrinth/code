import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import SegmentedControl from '../../components/base/SegmentedControl.vue'

const meta = {
	title: 'Base/SegmentedControl',
	// @ts-ignore - error comes from generically typed component
	component: SegmentedControl,
} satisfies Meta<typeof SegmentedControl>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		items: ['6h', '12h', '24h', '2 days', 'Weekly'],
		ariaLabel: 'Backup interval',
	},
	render: (args) => ({
		components: { SegmentedControl },
		setup() {
			const selected = ref('24h')
			return { args, selected }
		},
		template: '<SegmentedControl v-bind="args" v-model="selected" />',
	}),
}

export const FullWidth: Story = {
	...Default,
	args: { ...Default.args, fullWidth: true },
}

export const EqualWidth: Story = {
	...Default,
	args: {
		...Default.args,
		items: ['1h', '6h', '12h', '24h', '2 days', 'Weekly'],
		equalWidth: true,
	},
}

export const NeutralItems: Story = {
	args: {
		items: [true, false],
		ariaLabel: 'Fullscreen',
		formatLabel: (value: unknown) => (value ? 'On' : 'Off'),
		neutralItems: [false],
	},
	render: (args) => ({
		components: { SegmentedControl },
		setup() {
			const selected = ref(false)
			return { args, selected }
		},
		template: '<SegmentedControl v-bind="args" v-model="selected" />',
	}),
}

export const Disabled: Story = {
	...Default,
	args: { ...Default.args, disabled: true },
}
