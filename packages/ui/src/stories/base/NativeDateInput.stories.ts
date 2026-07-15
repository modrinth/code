import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import NativeDateInput from '../../components/base/inputs/NativeDateInput.vue'

const meta = {
	title: 'Base/NativeDateInput',
	component: NativeDateInput,
} satisfies Meta<typeof NativeDateInput>

export default meta
type Story = StoryObj<typeof meta>

export const Date: Story = {
	args: {
		type: 'date',
	},
}

export const DatetimeLocal: Story = {
	args: {
		type: 'datetime-local',
	},
}

export const Clearable: Story = {
	render: () => ({
		components: { NativeDateInput },
		setup() {
			const value = ref('2026-07-15')
			return { value }
		},
		template: '<NativeDateInput v-model="value" clearable />',
	}),
}

export const Error: Story = {
	args: {
		error: true,
	},
}
