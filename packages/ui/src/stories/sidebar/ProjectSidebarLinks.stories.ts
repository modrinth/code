import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarLinks from '../../components/project/ProjectSidebarLinks.vue'

const meta = {
	title: 'Sidebar/ProjectSidebarLinks',
	component: ProjectSidebarLinks,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 300px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectSidebarLinks>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		project: {
			issues_url: 'https://github.com/example/mod/issues',
			source_url: 'https://github.com/example/mod',
			wiki_url: 'https://wiki.example.com',
			discord_url: 'https://discord.gg/example',
			site_url: 'https://example.com',
			donation_urls: [
				{ id: 'patreon', url: 'https://patreon.com/example' },
				{ id: 'ko-fi', url: 'https://ko-fi.com/example' },
			],
		},
		linkTarget: '_blank',
	},
}

export const AllDonationTypes: StoryObj = {
	render: () => ({
		components: { ProjectSidebarLinks },
		template: /* html */ `
			<div style="max-width: 300px">
				<ProjectSidebarLinks
					:project="project"
					link-target="_blank"
				/>
			</div>
		`,
		setup() {
			return {
				project: {
					issues_url: 'https://github.com/example/mod/issues',
					source_url: 'https://github.com/example/mod',
					wiki_url: '',
					discord_url: '',
					donation_urls: [
						{ id: 'bmac', url: 'https://buymeacoffee.com/example' },
						{ id: 'patreon', url: 'https://patreon.com/example' },
						{ id: 'paypal', url: 'https://paypal.me/example' },
						{ id: 'ko-fi', url: 'https://ko-fi.com/example' },
						{ id: 'github', url: 'https://github.com/sponsors/example' },
						{ id: 'open-collective', url: 'https://opencollective.com/example' },
						{ id: 'other', url: 'https://example.com/donate' },
					],
				},
			}
		},
	}),
}

export const LinksOnly: Story = {
	args: {
		project: {
			issues_url: 'https://github.com/example/mod/issues',
			source_url: 'https://github.com/example/mod',
			wiki_url: 'https://wiki.example.com',
			discord_url: 'https://discord.gg/example',
			site_url: 'https://example.com',
			donation_urls: [],
		},
		linkTarget: '_blank',
	},
}

export const DonationsOnly: Story = {
	args: {
		project: {
			issues_url: '',
			source_url: '',
			wiki_url: '',
			discord_url: '',
			donation_urls: [
				{ id: 'patreon', url: 'https://patreon.com/example' },
				{ id: 'github', url: 'https://github.com/sponsors/example' },
			],
		},
		linkTarget: '_blank',
	},
}

export const MinimalLinks: Story = {
	args: {
		project: {
			issues_url: '',
			source_url: 'https://github.com/example/mod',
			wiki_url: '',
			discord_url: 'https://discord.gg/example',
			donation_urls: [],
		},
		linkTarget: '_blank',
	},
}
