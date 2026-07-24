import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import InlineEditableText from '../../components/base/InlineEditableText.vue'

const meta = {
	title: 'Base/InlineEditableText',
	component: InlineEditableText,
	render: (args) => ({
		components: { InlineEditableText },
		setup() {
			const value = ref('Editable text')
			return { args, value }
		},
		template: '<InlineEditableText v-model="value" v-bind="args" />',
	}),
	args: {
		editLabel: 'Edit text',
	},
} satisfies Meta<typeof InlineEditableText>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const ConstrainedWidth: Story = {
	args: {
		maxWidth: '12rem',
	},
	render: (args) => ({
		components: { InlineEditableText },
		setup() {
			const value = ref('A long value that truncates within its available width')
			return { args, value }
		},
		template: '<InlineEditableText v-model="value" v-bind="args" />',
	}),
}

export const Placeholder: Story = {
	args: {
		placeholder: 'Click to add a value',
	},
	render: (args) => ({
		components: { InlineEditableText },
		setup() {
			const value = ref('')
			return { args, value }
		},
		template: '<InlineEditableText v-model="value" v-bind="args" />',
	}),
}

export const DefaultValue: Story = {
	args: {
		defaultValue: 'Default value',
	},
	render: (args) => ({
		components: { InlineEditableText },
		setup() {
			const value = ref('')
			return { args, value }
		},
		template: '<InlineEditableText v-model="value" v-bind="args" />',
	}),
}

export const RejectedValue: Story = {
	args: {
		validate: (value: string) => value.toLowerCase() !== 'reserved',
	},
	render: (args) => ({
		components: { InlineEditableText },
		setup() {
			const value = ref('Try changing this to "reserved"')
			return { args, value }
		},
		template: '<InlineEditableText v-model="value" v-bind="args" />',
	}),
}
