import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectCard from '../../components/base/ProjectCard.vue'

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
		id: 'example-mod',
		type: 'mod',
		name: 'Example Mod',
		author: 'ModAuthor',
		description:
			'An example mod that demonstrates the ProjectCard component with a detailed description.',
		iconUrl: 'https://cdn.modrinth.com/data/AANobbMI/icon.png',
		downloads: '1234567',
		follows: '12345',
		createdAt: '2023-01-15T00:00:00Z',
		updatedAt: '2024-01-15T00:00:00Z',
		categories: ['adventure', 'decoration'],
		projectTypeDisplay: 'Mod',
		projectTypeUrl: 'mod',
		clientSide: 'required',
		serverSide: 'optional',
	},
}

export const AllTypes: Story = {
	render: () => ({
		components: { ProjectCard },
		template: `
			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<ProjectCard
					id="example-mod"
					type="mod"
					name="Example Mod"
					author="ModAuthor"
					description="A wonderful mod that adds new features to the game."
					downloads="1000000"
					follows="50000"
					createdAt="2023-01-15T00:00:00Z"
					:categories="['technology', 'magic']"
					projectTypeDisplay="Mod"
					projectTypeUrl="mod"
					clientSide="required"
					serverSide="optional"
				/>
				<ProjectCard
					id="example-plugin"
					type="plugin"
					name="Example Plugin"
					author="PluginDev"
					description="A server plugin for managing permissions."
					downloads="500000"
					follows="25000"
					createdAt="2023-06-01T00:00:00Z"
					:categories="['utility']"
					projectTypeDisplay="Plugin"
					projectTypeUrl="plugin"
					serverSide="required"
				/>
				<ProjectCard
					id="example-modpack"
					type="modpack"
					name="Example Modpack"
					author="PackCreator"
					description="A curated collection of mods for the best experience."
					downloads="250000"
					follows="10000"
					createdAt="2023-03-20T00:00:00Z"
					:categories="['adventure']"
					projectTypeDisplay="Modpack"
					projectTypeUrl="modpack"
				/>
				<ProjectCard
					id="example-resourcepack"
					type="resourcepack"
					name="HD Textures"
					author="ArtistName"
					description="High definition textures for a better visual experience."
					downloads="750000"
					follows="30000"
					createdAt="2022-12-01T00:00:00Z"
					:categories="['realistic']"
					projectTypeDisplay="Resource Pack"
					projectTypeUrl="resourcepack"
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
					id="draft-mod"
					type="mod"
					name="Draft Project"
					author="Developer"
					description="This project is still in draft mode."
					downloads="0"
					follows="0"
					createdAt="2024-01-01T00:00:00Z"
					:categories="['utility']"
					projectTypeDisplay="Mod"
					projectTypeUrl="mod"
					status="draft"
				/>
				<ProjectCard
					id="pending-mod"
					type="mod"
					name="Pending Review"
					author="Developer"
					description="This project is pending review."
					downloads="0"
					follows="0"
					createdAt="2024-01-01T00:00:00Z"
					:categories="['utility']"
					projectTypeDisplay="Mod"
					projectTypeUrl="mod"
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
							id="grid-mod"
							type="mod"
							name="Example Mod"
							author="ModAuthor"
							description="A wonderful mod that adds new features to the game."
							downloads="1000000"
							follows="50000"
							createdAt="2023-01-15T00:00:00Z"
							:categories="['technology', 'magic']"
							projectTypeDisplay="Mod"
							projectTypeUrl="mod"
							clientSide="required"
							serverSide="optional"
						/>
					</div>
				</div>
				<div>
					<h3 class="text-lg font-bold mb-4">List Mode</h3>
					<div class="display-mode--list">
						<ProjectCard
							id="list-mod"
							type="mod"
							name="Example Mod"
							author="ModAuthor"
							description="A wonderful mod that adds new features to the game."
							downloads="1000000"
							follows="50000"
							createdAt="2023-01-15T00:00:00Z"
							:categories="['technology', 'magic']"
							projectTypeDisplay="Mod"
							projectTypeUrl="mod"
							clientSide="required"
							serverSide="optional"
						/>
					</div>
				</div>
				<div>
					<h3 class="text-lg font-bold mb-4">Gallery Mode</h3>
					<div class="display-mode--gallery">
						<ProjectCard
							id="gallery-mod"
							type="mod"
							name="Example Mod"
							author="ModAuthor"
							description="A wonderful mod that adds new features to the game."
							downloads="1000000"
							follows="50000"
							createdAt="2023-01-15T00:00:00Z"
							:categories="['technology', 'magic']"
							projectTypeDisplay="Mod"
							projectTypeUrl="mod"
							clientSide="required"
							serverSide="optional"
							featuredImage="https://cdn.modrinth.com/data/AANobbMI/images/be1cc1abc9cd9c2f52bb6a39be0b4b05af24d813.png"
						/>
					</div>
				</div>
			</div>
		`,
	}),
}
