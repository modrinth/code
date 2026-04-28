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

export const CustomInputContent: Story = {
	args: {
		...Default.args,
		modelValue: ['en', 'es'],
		fitContent: true,
		showChevron: false,
		clearable: false,
		triggerClass:
			'h-10 max-w-[16rem] border border-solid border-surface-5 bg-surface-4 px-3 py-1.5 hover:bg-surface-5 hover:brightness-100 active:brightness-100',
	},
	render: (args) => ({
		components: { MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			const selectedLabel = () => {
				if (selected.value.length === 0) return 'All languages'
				if (selected.value.length === 1) {
					const option = args.options.find((item) => item.value === selected.value[0])
					return option?.label ?? '1 selected'
				}
				return `${selected.value.length} selected`
			}
			return { args, selected, selectedLabel }
		},
		template: /*html*/ `
			<div style="width: 400px;">
				<MultiSelect v-bind="args" v-model="selected">
					<template #input-content="{ isOpen }">
						<div class="flex min-w-0 items-center gap-2">
							<span class="truncate">
								<span class="font-medium">Languages:</span>
								<span class="ml-1 font-semibold text-contrast">{{ selectedLabel() }}</span>
							</span>
							<span
								class="text-secondary transition-transform duration-150"
								:style="{ transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)' }"
							>
								⌄
							</span>
							<button
								type="button"
								class="-mr-1 inline-flex size-5 shrink-0 items-center justify-center rounded-full border-0 bg-transparent text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
								@click.stop="selected = []"
							>
								×
							</button>
						</div>
					</template>
				</MultiSelect>
			</div>
		`,
	}),
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
