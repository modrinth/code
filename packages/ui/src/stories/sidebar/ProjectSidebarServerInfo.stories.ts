import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarServerInfo from '../../components/project/ProjectSidebarServerInfo.vue'

const gameVersions: GameVersionTag[] = [
	{ version: '1.21.4', version_type: 'release', date: '2024-12-03', major: true },
	{ version: '1.21.3', version_type: 'release', date: '2024-10-23', major: false },
	{ version: '1.21.1', version_type: 'release', date: '2024-08-08', major: true },
	{ version: '1.21', version_type: 'release', date: '2024-06-13', major: true },
	{ version: '1.20.4', version_type: 'release', date: '2023-12-07', major: true },
	{ version: '1.20.1', version_type: 'release', date: '2023-06-12', major: true },
]

const loaders: PlatformTag[] = [
	{ icon: '', name: 'fabric', supported_project_types: ['mod'] },
	{ icon: '', name: 'forge', supported_project_types: ['mod'] },
]

const tags = { gameVersions, loaders }

const meta = {
	title: 'Sidebar/ProjectSidebarServerInfo',
	component: ProjectSidebarServerInfo,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 300px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectSidebarServerInfo>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		projectV3: {
			minecraft_java_server: {
				address: 'play.example.com',
				content: {
					kind: 'vanilla',
					recommended_game_version: '1.21.4',
					supported_game_versions: ['1.21.4', '1.21.3', '1.21.1'],
				},
			},
		} as any,
		tags,
		ping: 42,
	},
}

export const WithRequiredContent: Story = {
	args: {
		projectV3: {
			minecraft_java_server: {
				address: 'mc.modrinth.com',
				content: {
					kind: 'modpack',
				},
			},
		} as any,
		tags,
		requiredContent: {
			name: 'Better MC [FABRIC] - BMC4',
			icon: 'https://cdn.modrinth.com/data/shrsKXYP/f68f3d07878e3cd26e33c1e379b85cdfc0e85a6d_96.webp',
		},
		recommendedVersion: '1.21.4',
		supportedVersions: ['1.21.4', '1.21.1', '1.20.4'],
		ping: 85,
	},
}

export const IPOnly: Story = {
	args: {
		projectV3: {
			minecraft_java_server: {
				address: 'play.hypixel.net',
				content: {
					kind: 'vanilla',
				},
			},
		} as any,
		tags,
	},
}

export const OfflineServer: Story = {
	args: {
		projectV3: {
			minecraft_java_server: {
				address: 'offline.example.com',
				content: {
					kind: 'vanilla',
					recommended_game_version: '1.21.4',
					supported_game_versions: ['1.21.4'],
				},
			},
		} as any,
		tags,
		ping: 0,
	},
}

export const HighLatency: Story = {
	args: {
		projectV3: {
			minecraft_java_server: {
				address: 'faraway.example.com',
				content: {
					kind: 'vanilla',
					recommended_game_version: '1.21.4',
					supported_game_versions: ['1.21.4', '1.21.1'],
				},
			},
		} as any,
		tags,
		ping: 350,
	},
}
