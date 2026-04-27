import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import DatePicker from '../../components/base/DatePicker.vue'

const meta = {
	title: 'Base/DatePicker',
	component: DatePicker,
} satisfies Meta<typeof DatePicker>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: (args) => ({
		components: { DatePicker },
		setup() {
			const value = ref('2026-04-27')
			return { args, value }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker v-model="value"
				wrapperClass="w-[300px]" v-bind="args" />
				<p class="text-sm text-secondary">Selected value: {{ value || 'None' }}</p>
			</div>
		`,
	}),
	args: {
		placeholder: 'Select a date...',
	},
}

export const WithTime: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref('2026-04-27 14:30')
			return { value }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker v-model="value"
				wrapperClass="w-[350px]" enable-time placeholder="Select a date and time..." />
				<p class="text-sm text-secondary">Selected value: {{ value || 'None' }}</p>
			</div>
		`,
	}),
}

export const Range: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref(['2026-04-27', '2026-05-04'])
			return { value }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker v-model="value"
				wrapperClass="w-[350px]" mode="range" placeholder="Select a date range..." />
				<p class="text-sm text-secondary">Selected value: {{ value?.join(' to ') || 'None' }}</p>
			</div>
		`,
	}),
}

export const MinMaxDates: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref('2026-04-27')
			return { value }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker
					v-model="value"
					wrapperClass="w-[350px]"
					min-date="2026-04-01"
					max-date="2026-04-30"
					placeholder="Select an April date..."
				/>
				<p class="text-sm text-secondary">Selected value: {{ value || 'None' }}</p>
			</div>
		`,
	}),
}

export const Disabled: Story = {
	args: {
		modelValue: '2026-04-27',
		disabled: true,
	},
}
