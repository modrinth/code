import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import MultiSelect from '../../components/base/MultiSelect.vue'

const meta = {
	title: 'Base/MultiSelect',
	// @ts-ignore - error comes from generically typed component
	component: MultiSelect,
	render: (args) => ({
		components: { MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			return { args, selected }
		},
		template: /*html*/ `
			<div style="width: 400px;">
				<MultiSelect v-bind="args" v-model="selected" />
			</div>
		`,
	}),
} satisfies Meta<typeof MultiSelect>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		options: [
			{ value: 'en', label: 'English' },
			{ value: 'es', label: 'Spanish' },
			{ value: 'fr', label: 'French' },
			{ value: 'de', label: 'German' },
			{ value: 'zh-CN', label: 'Chinese (Simplified)' },
			{ value: 'ko', label: 'Korean' },
			{ value: 'ja', label: 'Japanese' },
			{ value: 'pt', label: 'Portuguese' },
			{ value: 'ru', label: 'Russian' },
			{ value: 'it', label: 'Italian' },
			{ value: 'ar', label: 'Arabic' },
		],
		modelValue: ['en', 'es', 'fr', 'zh-CN'],
		placeholder: 'Select languages',
	},
}

export const WithSearch: Story = {
	args: {
		...Default.args,
		searchable: true,
		searchPlaceholder: 'Search versions',
	},
}

export const WithSelectAll: Story = {
	args: {
		...Default.args,
		searchable: true,
		includeSelectAllOption: true,
		searchPlaceholder: 'Search versions',
	},
}

export const ManySelected: Story = {
	args: {
		...Default.args,
		modelValue: ['en', 'es', 'fr', 'zh-CN', 'ko', 'ja', 'pt', 'ru', 'it', 'ar', 'de'],
		searchable: true,
		includeSelectAllOption: true,
	},
}

export const TwoTagRows: Story = {
	args: {
		...ManySelected.args,
		maxTagRows: 2,
	},
}

export const NoOptions: Story = {
	args: {
		...Default.args,
		options: [],
		modelValue: [],
		searchable: true,
		noOptionsMessage: 'No options available',
	},
}

export const Empty: Story = {
	args: {
		...Default.args,
		modelValue: [],
	},
}
