import type { Meta, StoryObj } from '@storybook/vue3-vite'

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
