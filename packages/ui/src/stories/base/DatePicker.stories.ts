import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, nextTick, onMounted, ref } from 'vue'

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

export const OpenWithTime: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref('2026-04-27 03:05')
			const datePicker = ref<InstanceType<typeof DatePicker> | null>(null)

			onMounted(async () => {
				await nextTick()
				datePicker.value?.open()
			})

			return { datePicker, value }
		},
		template: /* html */ `
			<div class="flex h-[460px] max-w-sm flex-col gap-2">
				<DatePicker
					ref="datePicker"
					v-model="value"
					wrapperClass="w-[350px]"
					enable-time
					placeholder="Select a date and time..."
				/>
				<p class="text-sm text-secondary">Selected value: {{ value || 'None' }}</p>
			</div>
		`,
	}),
}

export const Range: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref(['2026-04-13', '2026-04-29'])
			return { value }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker v-model="value"
				wrapperClass="w-[350px]" mode="range" default-view-date="2026-04-01" placeholder="Select a date range..." />
				<p class="text-sm text-secondary">Selected value: {{ value?.join(' to ') || 'None' }}</p>
			</div>
		`,
	}),
}

export const TwoMonthRange: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref(['2033-11-16', '2033-12-21'])
			const datePicker = ref<InstanceType<typeof DatePicker> | null>(null)

			onMounted(async () => {
				await nextTick()
				datePicker.value?.open()
			})

			return { datePicker, value }
		},
		template: /* html */ `
			<div class="flex h-[460px] max-w-[700px] flex-col gap-2">
				<DatePicker
					ref="datePicker"
					v-model="value"
					wrapperClass="w-[350px]"
					mode="range"
					:show-months="2"
					default-view-date="2033-11-01"
					placeholder="Select a date range..."
				/>
				<p class="text-sm text-secondary">Selected value: {{ value?.join(' to ') || 'None' }}</p>
			</div>
		`,
	}),
}

export const DraggableRange: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref(['2026-04-13', '2026-04-29'])
			const datePicker = ref<InstanceType<typeof DatePicker> | null>(null)

			onMounted(async () => {
				await nextTick()
				datePicker.value?.open()
			})

			return { datePicker, value }
		},
		template: /* html */ `
			<div class="flex h-[420px] max-w-sm flex-col gap-2">
				<DatePicker
					ref="datePicker"
					v-model="value"
					wrapperClass="w-[350px]"
					mode="range"
					default-view-date="2026-04-01"
					placeholder="Select a date range..."
				/>
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
			const datePicker = ref<InstanceType<typeof DatePicker> | null>(null)

			onMounted(async () => {
				await nextTick()
				datePicker.value?.open()
			})

			return { datePicker, value }
		},
		template: /* html */ `
			<div class="flex h-[420px] max-w-sm flex-col gap-2">
				<DatePicker
					ref="datePicker"
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

export const OpenCalendar: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref('2026-06-15')
			const datePicker = ref<InstanceType<typeof DatePicker> | null>(null)

			onMounted(async () => {
				await nextTick()
				datePicker.value?.open()
			})

			return { datePicker, value }
		},
		template: /* html */ `
			<div class="flex h-[420px] max-w-sm flex-col gap-2">
				<DatePicker
					ref="datePicker"
					v-model="value"
					wrapperClass="w-[300px]"
					default-view-date="2026-06-01"
				/>
				<p class="text-sm text-secondary">Selected value: {{ value || 'None' }}</p>
			</div>
		`,
	}),
}

export const PreserveDay: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref('2026-05-31')
			const intendedDay = ref<number | null>(null)
			const resolvedDay = ref<number | null>(null)
			const wasClamped = computed(
				() =>
					intendedDay.value !== null &&
					resolvedDay.value !== null &&
					intendedDay.value !== resolvedDay.value,
			)
			function onClamp(intended: number, resolved: number) {
				intendedDay.value = intended
				resolvedDay.value = resolved
			}
			return { value, intendedDay, resolvedDay, wasClamped, onClamp }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker
					v-model="value"
					wrapperClass="w-[300px]"
					preserve-day
					placeholder="Pick a date, then navigate months..."
					@clamp="onClamp"
				/>
				<p v-if="wasClamped" class="text-xs text-secondary">
					Day {{ intendedDay }} not available — showing {{ resolvedDay }}
				</p>
				<p class="text-sm text-secondary">Selected value: {{ value || 'None' }}</p>
				<p class="text-xs text-secondary">
					Try: pick May 31, then navigate to Feb (clamps to 28/29), then back to March (snaps to 31).
				</p>
			</div>
		`,
	}),
}

export const ShowToday: Story = {
	render: () => ({
		components: { DatePicker },
		setup() {
			const value = ref(null)
			return { value }
		},
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-2">
				<DatePicker v-model="value"
				wrapperClass="w-[300px]" show-today placeholder="Today is highlighted..." />
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
