import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarTags from '../../components/project/ProjectSidebarTags.vue'

const meta = {
	title: 'Project/ProjectSidebarTags',
	component: ProjectSidebarTags,
	render: (args) => ({
		components: { ProjectSidebarTags },
		setup() {
			return { args }
		},
		template: /* html */ `
			<ProjectSidebarTags v-bind="args" />
		`,
	}),
} satisfies Meta<typeof ProjectSidebarTags>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		project: {
			categories: ['adventure', 'technology', 'magic'],
			additional_categories: ['decoration', 'storage'],
		},
	},
}

export const CategoriesOnly: Story = {
	args: {
		project: {
			categories: ['adventure', 'technology'],
			additional_categories: [],
		},
	},
}

export const Empty: Story = {
	args: {
		project: {
			categories: [],
			additional_categories: [],
		},
	},
}
