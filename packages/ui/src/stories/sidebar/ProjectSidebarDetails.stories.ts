import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarDetails from '../../components/project/ProjectSidebarDetails.vue'

const now = new Date()
const daysAgo = (days: number) => new Date(now.getTime() - days * 86400000).toISOString()

const meta = {
	title: 'Sidebar/ProjectSidebarDetails',
	component: ProjectSidebarDetails,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 300px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectSidebarDetails>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		project: {
			id: 'project-1',
			published: daysAgo(365),
			updated: daysAgo(3),
			approved: daysAgo(360),
			queued: '',
			status: 'approved',
			license: {
				id: 'LGPL-3.0-only',
				url: 'https://www.gnu.org/licenses/lgpl-3.0.html',
			},
		},
		linkTarget: '_blank',
		hasVersions: true,
	},
}

export const AllRightsReserved: Story = {
	args: {
		project: {
			id: 'project-2',
			published: daysAgo(180),
			updated: daysAgo(7),
			approved: daysAgo(175),
			queued: '',
			status: 'approved',
			license: {
				id: 'LicenseRef-All-Rights-Reserved',
				url: '',
			},
		},
		linkTarget: '_blank',
		hasVersions: true,
	},
}

export const CustomLicense: Story = {
	args: {
		project: {
			id: 'project-3',
			published: daysAgo(90),
			updated: daysAgo(1),
			approved: daysAgo(85),
			queued: '',
			status: 'approved',
			license: {
				id: 'LicenseRef-My-Custom-License',
				url: 'https://example.com/license',
			},
		},
		linkTarget: '_blank',
		hasVersions: true,
	},
}

export const Processing: Story = {
	args: {
		project: {
			id: 'project-4',
			published: daysAgo(2),
			updated: '',
			approved: '',
			queued: daysAgo(1),
			status: 'processing',
			license: {
				id: 'MIT',
				url: 'https://opensource.org/licenses/MIT',
			},
		},
		linkTarget: '_blank',
		hasVersions: false,
	},
}

export const HiddenLicense: Story = {
	args: {
		project: {
			id: 'project-5',
			published: daysAgo(30),
			updated: daysAgo(5),
			approved: daysAgo(28),
			queued: '',
			status: 'approved',
			license: {
				id: 'MIT',
				url: '',
			},
		},
		linkTarget: '_blank',
		hasVersions: true,
		hideLicense: true,
	},
}
