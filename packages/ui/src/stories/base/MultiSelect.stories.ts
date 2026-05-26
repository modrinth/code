import { BoxIcon, CheckIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

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
	parameters: {
		docs: {
			description: {
				story:
					'Options render flush to the dropdown edges with full-width hover and selected states.',
			},
		},
	},
}

export const DeselectFocusState: Story = {
	args: {
		...Default.args,
		modelValue: ['en'],
	},
	parameters: {
		docs: {
			description: {
				story:
					'Mouse focus after deselecting an option should not keep the selected brightness state applied.',
			},
		},
	},
}

export const WithSearch: Story = {
	args: {
		...Default.args,
		searchable: true,
		searchPlaceholder: 'Search versions',
	},
	parameters: {
		docs: {
			description: {
				story:
					'Searchable dropdowns avoid auto-focusing search on mobile so opening the menu does not summon the soft keyboard.',
			},
		},
	},
}

export const WithOptionRightSlot: Story = {
	args: {
		options: [
			{ value: 'sodium-1.21.5', label: '1.21.5', searchTerms: ['Sodium'] },
			{ value: 'sodium-1.21.4', label: '1.21.4', searchTerms: ['Sodium'] },
			{ value: 'iris-1.20.1', label: '1.20.1', searchTerms: ['Iris'] },
			{ value: 'modmenu-1.19.2', label: '1.19.2', searchTerms: ['Mod Menu'] },
		],
		modelValue: ['sodium-1.21.5'],
		placeholder: 'Select versions',
		searchable: true,
		searchPlaceholder: 'Search versions',
	},
	render: (args) => ({
		components: { BoxIcon, MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			const projectNames: Record<string, string> = {
				'sodium-1.21.5': 'Sodium',
				'sodium-1.21.4': 'Sodium',
				'iris-1.20.1': 'Iris',
				'modmenu-1.19.2': 'Mod Menu',
			}
			return { args, projectNames, selected }
		},
		template: /*html*/ `
			<div style="width: 400px;">
				<MultiSelect v-bind="args" v-model="selected">
					<template #option-right="{ item }">
						<span
							v-tooltip="projectNames[item.value]"
							class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
						>
							<BoxIcon class="size-6" />
						</span>
					</template>
				</MultiSelect>
			</div>
		`,
	}),
}

export const WithSelectAll: Story = {
	args: {
		...Default.args,
		searchable: true,
		includeSelectAllOption: true,
		searchPlaceholder: 'Search versions',
	},
}

export const SingleOptionWithSelectAll: Story = {
	args: {
		options: [{ value: 'sodium', label: 'Sodium' }],
		modelValue: [],
		placeholder: 'Select projects',
		searchable: true,
		includeSelectAllOption: true,
		searchPlaceholder: 'Search projects',
	},
	parameters: {
		docs: {
			description: {
				story: 'Select all is hidden when there is only one enabled option.',
			},
		},
	},
}

export const WithRightCheckbox: Story = {
	args: {
		...Default.args,
		searchable: true,
		includeSelectAllOption: true,
		checkboxPosition: 'right',
		searchPlaceholder: 'Search languages',
	},
}

export const WithSelectionActions: Story = {
	args: {
		...Default.args,
		modelValue: [],
		searchable: true,
		showSelectionActions: true,
		searchPlaceholder: 'Search versions',
		maxHeight: 180,
	},
	parameters: {
		docs: {
			description: {
				story:
					'Selection actions stay above the scrollable options and compensate the scroll position when they appear.',
			},
		},
	},
}

export const WithSections: Story = {
	args: {
		options: [
			{ value: 'iris', label: 'Iris' },
			{ value: 'sodium', label: 'Sodium' },
			{ type: 'section-header', label: 'Single project group' },
			{ value: 'lithium', label: 'Lithium', searchTerms: ['Single project group'] },
			{ type: 'section-header', label: 'LambdAurora' },
			{ value: 'lambda-better-grass', label: 'LambdaBetterGrass', searchTerms: ['LambdAurora'] },
			{ value: 'auroras-decorations', label: "Aurora's Decorations", searchTerms: ['LambdAurora'] },
			{ type: 'section-header', label: 'Terraformers' },
			{ value: 'modmenu', label: 'Mod Menu', searchTerms: ['Terraformers'] },
			{ value: 'terraform-api', label: 'Terraform API', searchTerms: ['Terraformers'] },
		],
		modelValue: ['iris', 'modmenu'],
		placeholder: 'Select projects',
		searchable: true,
		showSelectionActions: true,
		searchPlaceholder: 'Search projects',
	},
}

export const WithTopSlot: Story = {
	args: {
		...Default.args,
		modelValue: [],
		searchable: true,
		showSelectionActions: true,
		searchPlaceholder: 'Search languages',
		placeholder: 'All languages',
	},
	render: (args) => ({
		components: { CheckIcon, MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			const isAllLanguagesSelected = computed(() => selected.value.length === 0)
			const selectAllLanguages = () => {
				selected.value = []
			}
			return { args, isAllLanguagesSelected, selectAllLanguages, selected }
		},
		template: /*html*/ `
			<div style="width: 400px;">
				<MultiSelect v-bind="args" v-model="selected">
					<template #top>
						<div class="px-3">
							<button
								type="button"
								class="flex w-full cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-transparent p-3 text-left text-contrast shadow-none transition-colors duration-150 hover:bg-surface-5 focus:bg-surface-5"
								:aria-selected="isAllLanguagesSelected"
								role="option"
								@click="selectAllLanguages"
								@keydown.enter.stop
								@keydown.space.stop
							>
								<span
									class="flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
									:class="
										isAllLanguagesSelected
											? 'border-button-border bg-brand text-brand-inverted'
											: 'border-surface-5 bg-surface-2'
									"
								>
									<CheckIcon v-if="isAllLanguagesSelected" aria-hidden="true" stroke-width="3" />
								</span>
								<span class="font-semibold leading-tight text-primary">All languages</span>
							</button>
						</div>
					</template>
				</MultiSelect>
			</div>
		`,
	}),
}

export const DropdownMinWidth: Story = {
	args: {
		...Default.args,
		modelValue: ['en'],
		dropdownMinWidth: 320,
		placeholder: 'Languages',
	},
	render: (args) => ({
		components: { MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			return { args, selected }
		},
		template: /*html*/ `
			<div style="width: 11rem;">
				<MultiSelect v-bind="args" v-model="selected" />
			</div>
		`,
	}),
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

export const WithBottomSlot: Story = {
	args: {
		...Default.args,
		modelValue: ['en', 'es'],
		searchable: true,
		includeSelectAllOption: true,
	},
	render: (args) => ({
		components: { MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			const minimum = ref('')
			return { args, selected, minimum }
		},
		template: /*html*/ `
			<div style="width: 400px;">
				<MultiSelect v-bind="args" v-model="selected">
					<template #bottom>
						<div style="display: flex; align-items: center; gap: 0.75rem; border-top: 1px solid var(--color-surface-5); padding: 0.75rem;">
							<span style="font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary);">Projects above</span>
							<input
								v-model="minimum"
								type="text"
								style="height: 2rem; width: 5rem; border: 1px solid var(--color-surface-5); border-radius: 0.5rem; background: var(--color-surface-3); color: var(--color-text-primary); text-align: center; font-weight: 600;"
							/>
							<span style="font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary);">downloads</span>
						</div>
					</template>
				</MultiSelect>
			</div>
		`,
	}),
}

export const VirtualizedLargeList: Story = {
	args: {
		options: Array.from({ length: 250 }, (_, index) => {
			const version = `1.${Math.floor(index / 10) + 1}.${index % 10}`
			return {
				value: `version-${index + 1}`,
				label: version,
				searchTerms: [`Project ${Math.floor(index / 25) + 1}`],
			}
		}),
		modelValue: ['version-3', 'version-47', 'version-132'],
		placeholder: 'Select versions',
		searchable: true,
		searchPlaceholder: 'Search versions',
		showSelectionActions: true,
		maxHeight: 320,
	},
	render: (args) => ({
		components: { BoxIcon, MultiSelect },
		setup() {
			const selected = ref(args.modelValue)
			function getProjectName(value: string) {
				const optionIndex = Number(value.replace('version-', '')) - 1
				return `Project ${Math.floor(optionIndex / 25) + 1}`
			}
			return { args, getProjectName, selected }
		},
		template: /*html*/ `
			<div style="width: 400px;">
				<MultiSelect v-bind="args" v-model="selected">
					<template #option-right="{ item }">
						<span
							v-tooltip="getProjectName(item.value)"
							class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
						>
							<BoxIcon class="size-6" />
						</span>
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
