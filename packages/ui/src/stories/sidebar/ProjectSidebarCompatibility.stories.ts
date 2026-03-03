import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarCompatibility from '../../components/project/ProjectSidebarCompatibility.vue'

const gameVersions: GameVersionTag[] = [
	{ version: '1.21.4', version_type: 'release', date: '2024-12-03', major: true },
	{ version: '1.21.3', version_type: 'release', date: '2024-10-23', major: false },
	{ version: '1.21.2', version_type: 'release', date: '2024-09-18', major: false },
	{ version: '1.21.1', version_type: 'release', date: '2024-08-08', major: true },
	{ version: '1.21', version_type: 'release', date: '2024-06-13', major: true },
	{ version: '1.20.6', version_type: 'release', date: '2024-04-29', major: false },
	{ version: '1.20.4', version_type: 'release', date: '2023-12-07', major: true },
	{ version: '1.20.2', version_type: 'release', date: '2023-09-21', major: true },
	{ version: '1.20.1', version_type: 'release', date: '2023-06-12', major: true },
	{ version: '1.20', version_type: 'release', date: '2023-06-07', major: true },
]

const loaders: PlatformTag[] = [
	{ icon: '', name: 'fabric', supported_project_types: ['mod'] },
	{ icon: '', name: 'forge', supported_project_types: ['mod'] },
	{ icon: '', name: 'neoforge', supported_project_types: ['mod'] },
	{ icon: '', name: 'quilt', supported_project_types: ['mod'] },
]

const meta = {
	title: 'Sidebar/ProjectSidebarCompatibility',
	component: ProjectSidebarCompatibility,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 300px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectSidebarCompatibility>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		project: {
			actualProjectType: 'mod',
			project_type: 'mod',
			loaders: ['fabric', 'forge', 'neoforge'],
			client_side: 'required',
			server_side: 'optional',
			versions: [{ game_versions: ['1.21.4', '1.21.3', '1.21.1', '1.20.4', '1.20.1'] }],
		},
		tags: {
			gameVersions,
			loaders,
		},
	},
}

export const ClientSideOnly: StoryObj = {
	render: () => ({
		components: { ProjectSidebarCompatibility },
		template: /* html */ `
			<div style="max-width: 300px">
				<ProjectSidebarCompatibility
					:project="{
						actualProjectType: 'mod',
						project_type: 'mod',
						loaders: ['fabric'],
						client_side: 'required',
						server_side: 'unsupported',
						versions: [{ game_versions: ['1.21.4', '1.21.1'] }],
					}"
					:tags="tags"
				/>
			</div>
		`,
		setup() {
			return { tags: { gameVersions, loaders } }
		},
	}),
}

export const ServerSideOnly: StoryObj = {
	render: () => ({
		components: { ProjectSidebarCompatibility },
		template: /* html */ `
			<div style="max-width: 300px">
				<ProjectSidebarCompatibility
					:project="{
						actualProjectType: 'mod',
						project_type: 'mod',
						loaders: ['fabric', 'forge'],
						client_side: 'unsupported',
						server_side: 'required',
						versions: [{ game_versions: ['1.20.1', '1.20.4'] }],
					}"
					:tags="tags"
				/>
			</div>
		`,
		setup() {
			return { tags: { gameVersions, loaders } }
		},
	}),
}

export const ResourcePack: StoryObj = {
	render: () => ({
		components: { ProjectSidebarCompatibility },
		template: /* html */ `
			<div style="max-width: 300px">
				<ProjectSidebarCompatibility
					:project="{
						actualProjectType: 'resourcepack',
						project_type: 'resourcepack',
						loaders: ['minecraft'],
						client_side: 'required',
						server_side: 'unsupported',
						versions: [{ game_versions: ['1.21.4', '1.21.3', '1.21.1', '1.21', '1.20.6', '1.20.4'] }],
					}"
					:tags="tags"
				/>
			</div>
		`,
		setup() {
			return { tags: { gameVersions, loaders } }
		},
	}),
}

export const Modpack: StoryObj = {
	render: () => ({
		components: { ProjectSidebarCompatibility },
		template: /* html */ `
			<div style="max-width: 300px">
				<ProjectSidebarCompatibility
					:project="{
						actualProjectType: 'modpack',
						project_type: 'modpack',
						loaders: ['fabric'],
						client_side: 'required',
						server_side: 'required',
						versions: [{ game_versions: ['1.21.4'] }],
					}"
					:tags="tags"
				/>
			</div>
		`,
		setup() {
			return { tags: { gameVersions, loaders } }
		},
	}),
}
