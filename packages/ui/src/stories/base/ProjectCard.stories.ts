import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectCard from '../../components/project/card/ProjectCard.vue'

const meta = {
	title: 'Base/ProjectCard',
	component: ProjectCard,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="display-mode--grid"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectCard>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		link: '/mod/example-mod',
		layout: 'grid',
		title: 'Example Mod',
		author: { name: 'Prospector', link: 'https://modrinth.com/user/Prospector' },
		summary:
			'An example mod that demonstrates the ProjectCard component with a detailed description.',
		iconUrl: 'https://cdn.modrinth.com/data/AANobbMI/icon.png',
		downloads: 1234567,
		followers: 12345,
		dateUpdated: '2024-01-15T00:00:00Z',
		tags: ['adventure', 'decoration'],
		environment: {
			clientSide: 'required',
			serverSide: 'optional',
		},
	},
}

export const AllTypes: Story = {
	render: () => ({
		components: { ProjectCard },
		template: `
			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<ProjectCard
					link="/mod/example-mod"
					layout="grid"
					title="Example Mod"
					:author="{ name: 'ModAuthor', link: '/user/ModAuthor' }"
					summary="A wonderful mod that adds new features to the game."
					:downloads="1000000"
					:followers="50000"
					date-updated="2023-01-15T00:00:00Z"
					:tags="['technology', 'magic']"
					:environment="{ clientSide: 'required', serverSide: 'optional' }"
				/>
				<ProjectCard
					link="/plugin/example-plugin"
					layout="grid"
					title="Example Plugin"
					:author="{ name: 'PluginDev', link: '/user/PluginDev' }"
					summary="A server plugin for managing permissions."
					:downloads="500000"
					:followers="25000"
					date-updated="2023-06-01T00:00:00Z"
					:tags="['utility']"
					:environment="{ clientSide: 'unsupported', serverSide: 'required' }"
				/>
				<ProjectCard
					link="/modpack/example-modpack"
					layout="grid"
					title="Example Modpack"
					:author="{ name: 'PackCreator', link: '/user/PackCreator' }"
					summary="A curated collection of mods for the best experience."
					:downloads="250000"
					:followers="10000"
					date-updated="2023-03-20T00:00:00Z"
					:tags="['adventure']"
				/>
				<ProjectCard
					link="/resourcepack/example-resourcepack"
					layout="grid"
					title="HD Textures"
					:author="{ name: 'ArtistName', link: '/user/ArtistName' }"
					summary="High definition textures for a better visual experience."
					:downloads="750000"
					:followers="30000"
					date-updated="2022-12-01T00:00:00Z"
					:tags="['realistic']"
				/>
			</div>
		`,
	}),
}

export const WithStatus: Story = {
	render: () => ({
		components: { ProjectCard },
		template: `
			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<ProjectCard
					link="/mod/draft-mod"
					layout="grid"
					title="Draft Project"
					:author="{ name: 'Developer', link: '/user/Developer' }"
					summary="This project is still in draft mode."
					:downloads="0"
					:followers="0"
					date-updated="2024-01-01T00:00:00Z"
					:tags="['utility']"
					status="draft"
				/>
				<ProjectCard
					link="/mod/pending-mod"
					layout="grid"
					title="Pending Review"
					:author="{ name: 'Developer', link: '/user/Developer' }"
					summary="This project is pending review."
					:downloads="0"
					:followers="0"
					date-updated="2024-01-01T00:00:00Z"
					:tags="['utility']"
					status="processing"
				/>
			</div>
		`,
	}),
}

export const DisplayModes: StoryObj = {
	decorators: [], // Remove default decorator for this story
	render: () => ({
		components: { ProjectCard },
		template: `
			<div class="flex flex-col gap-8">
				<div>
					<h3 class="text-lg font-bold mb-4">Grid Mode</h3>
					<div class="display-mode--grid">
						<ProjectCard
							link="/mod/grid-mod"
							layout="grid"
							title="Example Mod"
							:author="{ name: 'ModAuthor', link: '/user/ModAuthor' }"
							summary="A wonderful mod that adds new features to the game."
							:downloads="1000000"
							:followers="50000"
							date-updated="2023-01-15T00:00:00Z"
							:tags="['technology', 'magic']"
							:environment="{ clientSide: 'required', serverSide: 'optional' }"
						/>
					</div>
				</div>
				<div>
					<h3 class="text-lg font-bold mb-4">List Mode</h3>
					<div class="display-mode--list">
						<ProjectCard
							link="/mod/list-mod"
							layout="list"
							title="Example Mod"
							:author="{ name: 'ModAuthor', link: '/user/ModAuthor' }"
							summary="A wonderful mod that adds new features to the game."
							:downloads="1000000"
							:followers="50000"
							date-updated="2023-01-15T00:00:00Z"
							:tags="['technology', 'magic']"
							:environment="{ clientSide: 'required', serverSide: 'optional' }"
						/>
					</div>
				</div>
				<div>
					<h3 class="text-lg font-bold mb-4">Grid with Banner</h3>
					<div class="display-mode--grid">
						<ProjectCard
							link="/mod/gallery-mod"
							layout="grid"
							title="Example Mod"
							:author="{ name: 'ModAuthor', link: '/user/ModAuthor' }"
							summary="A wonderful mod that adds new features to the game."
							:downloads="1000000"
							:followers="50000"
							date-updated="2023-01-15T00:00:00Z"
							:tags="['technology', 'magic']"
							:environment="{ clientSide: 'required', serverSide: 'optional' }"
							banner="https://cdn.modrinth.com/data/AANobbMI/images/be1cc1abc9cd9c2f52bb6a39be0b4b05af24d813.png"
						/>
					</div>
				</div>
			</div>
		`,
	}),
}
