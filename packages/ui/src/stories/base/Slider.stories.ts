import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Slider from '../../components/base/Slider.vue'

const meta = {
	title: 'Base/Slider',
	component: Slider,
} satisfies Meta<typeof Slider>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		modelValue: 50,
		min: 0,
		max: 100,
	},
}

export const WithUnit: Story = {
	args: {
		modelValue: 50,
		min: 0,
		max: 100,
		unit: '%',
	},
}

export const WithSnapPoints: Story = {
	args: {
		modelValue: 25,
		min: 0,
		max: 100,
		step: 25,
		snapPoints: [0, 25, 50, 75, 100],
	},
}

export const Disabled: Story = {
	args: {
		modelValue: 50,
		min: 0,
		max: 100,
		disabled: true,
	},
}
