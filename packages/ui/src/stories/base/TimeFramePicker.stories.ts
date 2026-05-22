import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import TimeFramePicker, {
	type TimeFrameLastUnit,
	type TimeFrameMode,
	type TimeFramePreset,
} from '../../components/base/TimeFramePicker.vue'

const meta = {
	title: 'Base/TimeFramePicker',
	component: TimeFramePicker,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="width: 20rem;"><story /></div>',
		}),
	],
} satisfies Meta<typeof TimeFramePicker>

export default meta
type Story = StoryObj<typeof meta>

function renderPicker(
	initial: {
		mode?: TimeFrameMode
		preset?: TimeFramePreset
		lastAmount?: number
		lastUnit?: TimeFrameLastUnit
		customStartDate?: string
		customEndDate?: string
	} = {},
	triggerClass?: string,
) {
	return () => ({
		components: { TimeFramePicker },
		setup() {
			const mode = ref<TimeFrameMode>(initial.mode ?? 'preset')
			const preset = ref<TimeFramePreset>(initial.preset ?? 'last_30_days')
			const lastAmount = ref(initial.lastAmount ?? 1)
			const lastUnit = ref<TimeFrameLastUnit>(initial.lastUnit ?? 'days')
			const customStartDate = ref(initial.customStartDate ?? '2026-04-23')
			const customEndDate = ref(initial.customEndDate ?? '2026-05-22')

			return {
				customEndDate,
				customStartDate,
				lastAmount,
				lastUnit,
				mode,
				preset,
				triggerClass,
			}
		},
		template: /* html */ `
			<TimeFramePicker
				v-model:mode="mode"
				v-model:preset="preset"
				v-model:last-amount="lastAmount"
				v-model:last-unit="lastUnit"
				v-model:custom-start-date="customStartDate"
				v-model:custom-end-date="customEndDate"
				:trigger-class="triggerClass"
				min-date="2023-01-01"
			/>
		`,
	})
}

export const Preset: Story = {
	render: renderPicker(),
}

export const LastTimeframe: Story = {
	render: renderPicker({
		mode: 'last',
		lastAmount: 12,
		lastUnit: 'hours',
	}),
}

export const CustomRange: Story = {
	render: renderPicker({
		mode: 'custom_range',
		customStartDate: '2026-04-23',
		customEndDate: '2026-05-22',
	}),
}

export const CompactTrigger: Story = {
	render: renderPicker(
		{},
		'!h-10 !min-h-10 !rounded-[14px] !bg-surface-4 !py-2.5 !text-base shadow-[0px_1px_1px_rgba(0,0,0,0.3),0px_1px_1.5px_rgba(0,0,0,0.15)]',
	),
}
