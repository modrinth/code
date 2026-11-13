import type { Meta, StoryObj } from '@storybook/vue3-vite'

import TagItem from '../../components/base/TagItem.vue'
import ProjectCombobox from '../../components/project/ProjectCombobox.vue'

const meta = {
	title: 'Project/ProjectCombobox',
	component: ProjectCombobox,
} satisfies Meta<typeof ProjectCombobox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		placeholder: 'Select project',
		searchPlaceholder: 'Search by name or paste ID...',
		loadingMessage: 'Loading...',
		noResultsMessage: 'No results found',
		disabled: false,
		limit: 20,
	},
}

export const SearchByProjectId: Story = {
	render: (args) => ({
		components: { ProjectCombobox, TagItem },
		setup: () => ({ args }),
		template: `
			<ProjectCombobox v-bind="args">
				<template #option-suffix="{ item }">
					<TagItem class="shrink-0 font-mono">{{ item.value }}</TagItem>
				</template>
			</ProjectCombobox>
		`,
	}),
	args: {
		searchPlaceholder: 'Paste project id or search projects...',
		lookupProjectIds: true,
		clearable: true,
	},
}
