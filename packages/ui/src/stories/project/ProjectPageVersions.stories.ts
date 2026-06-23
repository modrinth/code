import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, MoreVerticalIcon } from '@modrinth/assets'
import type { GameVersionTag, Version } from '@modrinth/utils'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ProjectPageVersions from '../../components/project/ProjectPageVersions.vue'

type StoryVersion = Version & {
	displayUrlEnding: string
	environment?: Labrinth.Projects.v3.Environment
	mrpack_loaders?: string[]
	files_missing_attribution?: string[]
}

const gameVersions: GameVersionTag[] = [
	{ version: '1.21.4', version_type: 'release', date: '2024-12-03', major: true },
	{ version: '1.21.3', version_type: 'release', date: '2024-10-23', major: false },
	{ version: '1.21.1', version_type: 'release', date: '2024-08-08', major: true },
	{ version: '1.20.6', version_type: 'release', date: '2024-04-29', major: false },
	{ version: '1.20.4', version_type: 'release', date: '2023-12-07', major: true },
	{ version: '1.20.1', version_type: 'release', date: '2023-06-12', major: true },
	{ version: '1.19.4', version_type: 'release', date: '2023-03-14', major: true },
]

const loaders: Labrinth.Tags.v2.Loader[] = [
	{ icon: '', name: 'fabric', supported_project_types: ['mod', 'modpack'] },
	{ icon: '', name: 'forge', supported_project_types: ['mod', 'modpack'] },
	{ icon: '', name: 'neoforge', supported_project_types: ['mod', 'modpack'] },
	{ icon: '', name: 'quilt', supported_project_types: ['mod', 'modpack'] },
]

const versions: StoryVersion[] = [
	{
		id: 'version-1',
		project_id: 'project-1',
		author_id: 'author-1',
		name: 'Performance improvements and bug fixes',
		version_number: 'mc1.21.4-0.6.13',
		displayUrlEnding: 'mc1.21.4-0.6.13',
		changelog: '',
		dependencies: [],
		game_versions: ['1.21.4', '1.21.3', '1.21.1'],
		version_type: 'release',
		loaders: ['fabric', 'quilt'],
		featured: true,
		status: 'listed',
		date_published: new Date(Date.now() - 1000 * 60 * 60 * 24 * 2).toISOString(),
		downloads: 1258400,
		environment: 'client_only',
		files: [
			{
				hashes: { sha512: 'sha512-1', sha1: 'sha1-1' },
				url: 'https://cdn.modrinth.com/data/story/version-1.jar',
				filename: 'sodium-fabric-0.6.13.jar',
				primary: true,
				size: 1248200,
			},
		],
	},
	{
		id: 'version-2',
		project_id: 'project-1',
		author_id: 'author-1',
		name: 'NeoForge compatibility',
		version_number: 'mc1.20.6-0.5.11',
		displayUrlEnding: 'mc1.20.6-0.5.11',
		changelog: '',
		dependencies: [],
		game_versions: ['1.20.6', '1.20.4', '1.20.1', '1.19.4'],
		version_type: 'beta',
		loaders: ['neoforge'],
		featured: false,
		status: 'listed',
		date_published: new Date(Date.now() - 1000 * 60 * 60 * 24 * 16).toISOString(),
		downloads: 84200,
		environment: 'client_and_server',
		files_missing_attribution: ['bundled-library.jar'],
		files: [
			{
				hashes: { sha512: 'sha512-2', sha1: 'sha1-2' },
				url: 'https://cdn.modrinth.com/data/story/version-2.jar',
				filename: 'sodium-neoforge-0.5.11.jar',
				primary: true,
				size: 1424200,
			},
			{
				hashes: { sha512: 'sha512-3', sha1: 'sha1-3' },
				url: 'https://cdn.modrinth.com/data/story/version-2-sources.jar',
				filename: 'sodium-neoforge-0.5.11-sources.jar',
				primary: false,
				size: 624200,
				file_type: 'sources-jar',
			},
		],
	},
	{
		id: 'version-3',
		project_id: 'project-1',
		author_id: 'author-1',
		name: 'Server pack with no mod loader',
		version_number: 'server-pack-1.0.0',
		displayUrlEnding: 'server-pack-1.0.0',
		changelog: '',
		dependencies: [],
		game_versions: ['1.21.4'],
		version_type: 'alpha',
		loaders: ['minecraft'],
		mrpack_loaders: [],
		featured: false,
		status: 'listed',
		date_published: new Date(Date.now() - 1000 * 60 * 60 * 24 * 45).toISOString(),
		downloads: 1200,
		environment: 'server_only',
		files: [
			{
				hashes: { sha512: 'sha512-4', sha1: 'sha1-4' },
				url: 'https://cdn.modrinth.com/data/story/version-3.mrpack',
				filename: 'server-pack-1.0.0.mrpack',
				primary: true,
				size: 2048200,
			},
		],
	},
]

const meta = {
	title: 'Project/ProjectPageVersions',
	component: ProjectPageVersions,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="p-4"><story /></div>',
		}),
	],
	args: {
		project: {
			project_type: 'mod',
			slug: 'sodium',
			id: 'project-1',
		},
		versions,
		loaders,
		gameVersions,
		baseId: 'project-page-versions-story',
		showFiles: false,
		showEnvironmentColumn: false,
		versionLink: (version: Version) => `https://modrinth.com/mod/sodium/version/${version.id}`,
	},
	render: (args) => ({
		components: { ButtonStyled, DownloadIcon, MoreVerticalIcon, ProjectPageVersions },
		setup() {
			return { args }
		},
		template: /* html */ `
			<ProjectPageVersions v-bind="args">
				<template #actions="{ version }">
					<ButtonStyled circular type="transparent">
						<a
							v-tooltip="'Download'"
							:href="version.files[0]?.url"
							:download="version.files[0]?.filename"
							aria-label="Download"
						>
							<DownloadIcon aria-hidden="true" />
						</a>
					</ButtonStyled>
					<ButtonStyled circular type="transparent">
						<button v-tooltip="'More options'" aria-label="More options">
							<MoreVerticalIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</template>
			</ProjectPageVersions>
		`,
	}),
} satisfies Meta<typeof ProjectPageVersions>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithFiles: Story = {
	args: {
		showFiles: true,
	},
}

export const WithEnvironmentColumn: Story = {
	args: {
		showEnvironmentColumn: true,
	},
}

export const MobileWidth: Story = {
	parameters: {
		viewport: {
			defaultViewport: 'mobile1',
		},
	},
	args: {
		showFiles: true,
		showEnvironmentColumn: true,
	},
}
