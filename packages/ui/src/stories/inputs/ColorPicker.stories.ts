import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ColorPicker from '../../components/base/inputs/ColorPicker.vue'

const meta = {
	title: 'Inputs/ColorPicker',
	component: ColorPicker,
} satisfies Meta<typeof ColorPicker>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: (args) => ({
		components: { ColorPicker },
		setup() {
			const value = ref('#ff6b6b')
			return { args, value }
		},
		template: /* html */ `
			<div class="flex flex-col gap-2">
				<ColorPicker v-model="value" v-bind="args" />
				<p class="text-sm text-secondary">Selected value: {{ value }}</p>
			</div>
		`,
	}),
	args: {
		label: 'Colour',
	},
}

export const Small: Story = {
	render: () => ({
		components: { ColorPicker },
		setup() {
			const value = ref('#3b82f6')
			return { value }
		},
		template: /* html */ `
			<div class="flex flex-col gap-2">
				<ColorPicker v-model="value" label="Colour" size="sm" />
				<p class="text-sm text-secondary">Selected value: {{ value }}</p>
			</div>
		`,
	}),
}

export const Disabled: Story = {
	args: {
		label: 'Colour',
		modelValue: '#22c55e',
		disabled: true,
	},
}

export const FocusAndChangeEvents: Story = {
	render: () => ({
		components: { ColorPicker },
		setup() {
			const value = ref('#a855f7')
			const log = ref<string[]>([])
			return { value, log }
		},
		template: /* html */ `
			<div class="flex flex-col gap-2">
				<ColorPicker
					v-model="value"
					label="Colour"
					@focus="log.unshift('focus: edit session started')"
					@change="(next) => log.unshift('change: committed ' + next)"
				/>
				<ul class="text-sm text-secondary">
					<li v-for="(entry, index) in log" :key="index">{{ entry }}</li>
				</ul>
			</div>
		`,
	}),
}
