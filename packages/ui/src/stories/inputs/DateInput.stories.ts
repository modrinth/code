import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import DateInput from '../../components/base/inputs/DateInput.vue'

const meta = {
	title: 'Inputs/DateInput',
	component: DateInput,
} satisfies Meta<typeof DateInput>

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
		components: { DateInput },
		setup() {
			const value = ref('2026-07-15')
			return { value }
		},
		template: '<DateInput v-model="value" clearable />',
	}),
}

export const Error: Story = {
	args: {
		error: true,
	},
}
