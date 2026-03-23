import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarCreators from '../../components/project/ProjectSidebarCreators.vue'

const mockMembers = [
	{
		id: 'member-1',
		role: 'Owner',
		is_owner: true,
		accepted: true,
		user: {
			id: 'user-1',
			username: 'jellysquid3',
			avatar_url: 'https://avatars.githubusercontent.com/u/31803019?v=4',
		},
	},
	{
		id: 'member-2',
		role: 'Developer',
		is_owner: false,
		accepted: true,
		user: {
			id: 'user-2',
			username: 'modder42',
			avatar_url: '',
		},
	},
	{
		id: 'member-3',
		role: 'Artist',
		is_owner: false,
		accepted: true,
		user: {
			id: 'user-3',
			username: 'pixelartist',
			avatar_url: '',
		},
	},
]

const mockOrganization = {
	id: 'org-1',
	slug: 'caffeine-mc',
	name: 'CaffeineMC',
	icon_url: 'https://avatars.githubusercontent.com/u/74333534?v=4',
	avatar_url: 'https://avatars.githubusercontent.com/u/74333534?v=4',
	members: [
		{
			id: 'member-1',
			role: 'Owner',
			is_owner: true,
			accepted: true,
			user: {
				id: 'user-1',
				username: 'jellysquid3',
				avatar_url: 'https://avatars.githubusercontent.com/u/31803019?v=4',
			},
		},
	],
}

const noopLink = (s: string) => `/${s}`

const meta = {
	title: 'Sidebar/ProjectSidebarCreators',
	component: ProjectSidebarCreators,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 300px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectSidebarCreators>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		members: mockMembers,
		orgLink: noopLink,
		userLink: (username: string) => `/user/${username}`,
	},
}

export const WithOrganization: Story = {
	args: {
		members: mockMembers,
		organization: mockOrganization,
		orgLink: (slug: string) => `/organization/${slug}`,
		userLink: (username: string) => `/user/${username}`,
	},
}

export const SingleMember: Story = {
	args: {
		members: [mockMembers[0]],
		orgLink: noopLink,
		userLink: (username: string) => `/user/${username}`,
	},
}

export const ExternalLinks: Story = {
	args: {
		members: mockMembers,
		organization: mockOrganization,
		orgLink: (slug: string) => `/organization/${slug}`,
		userLink: (username: string) => `/user/${username}`,
		linkTarget: '_blank',
	},
}
