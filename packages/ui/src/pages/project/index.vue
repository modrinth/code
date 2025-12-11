<script setup lang="ts">
import { AsteriskIcon,ImageIcon, InfoIcon } from '@modrinth/assets'
import { computed } from 'vue'

import {
	Avatar,
	ContentPageHeader,
	LargeCopyField,
	NavTabs,
	NormalPage,
	SidebarCard,
} from '../../components'

const project = {
	id: 'bJuQ1kl5',
	slug: 'complex-cobblemon',
	project_types: [
		'server', // add server type
		'modpack',
	],
	games: ['minecraft-java'],
	team_id: 'aUYO8LVq',
	organization: null,
	name: 'Hypixel',
	summary: 'A mini-game network, featuring Bed Wars, SkyBlock, SkyWars, Murder Mystery, and more!',
	description: 'project desc',
	published: '2025-12-07T20:07:01.351121Z',
	updated: '2025-12-07T22:31:02.164985Z', // should track when server IP, mc version, or content changes
	approved: '2025-12-08T03:09:10.517396Z',
	queued: '2025-12-07T21:50:54.823172Z',
	status: 'approved',
	requested_status: 'approved',
	moderator_message: null,
	license: null, // becomes nullable
	downloads: 1, // dunno if we should show? could count total server pack downloads or just always 0/null
	followers: 0,
	categories: ['game-mechanics', 'utility'], // tbd, but will be server-specific categories
	additional_categories: [],
	loaders: null, // nullable or empty, depending on what is possible
	versions: ['dA2xTlv'], // only show the current active release
	icon_url: 'https://avatars.githubusercontent.com/u/3840546?s=280&v=4',
	link_urls: {}, // same as projects, maybe allow some extra types? can figure out later
	gallery: [],
	color: 12553338,
	thread_id: 'ML721Sj1',
	monetization_status: null, // nullable or empty, depending on what is possible
	side_types_migration_review_status: null, // nullable or empty, depending on what is possible
	game_versions: null, // nullable or empty, depending on what is possible
	environment: null, // nullable or empty, depending on what is possible
	server_metadata: {
		// always exists when project_types contains "server", otherwise does not
		java_address: 'mc.hypixel.net',
		bedrock_address: null,
		players_online: 1234,
		max_players: 20000, // nullable, allow creators to show no max
		plays: 0, // number of times people have joined the server via Modrinth App
		total_players: 2304, // number of unique players who have played via Modrinth App
		content: {
			// IF vanilla
			type: 'vanilla',
			suggested_minecraft_version: '1.21.10',
			supported_minecraft_versions: ['1.21.9', '1.21.10'],
		},
		// content: {
		// 	// ELSE IF modpack
		// 	type: 'modpack',
		// 	modpack_version_id: 'dA2xTlv',
		// },
	},
}

const server = computed(() =>
	project.project_types.includes('server') ? project.server_metadata : false,
)

const tabs = computed(() => {})
</script>
<template>
	<NormalPage sidebar="right">
		<template #header>
			<ContentPageHeader>
				<template #icon>
					<Avatar :src="project.icon_url" size="104px" />
				</template>
				<template #title>
					{{ project.name }}
				</template>
				<template #summary>
					{{ project.summary }}
				</template>
			</ContentPageHeader>
			<NavTabs
				:links="[
					{
						label: 'Description',
						href: '',
						icon: InfoIcon,
					},
					{
						label: 'Gallery',
						href: '',
						icon: ImageIcon,
					},
					{
						label: 'Required content',
						href: '',
						icon: AsteriskIcon,
					},
				]"
				class="mt-6"
			/>
		</template>
		<template v-if="server" #sidebar>
			<SidebarCard>
				<LargeCopyField :text="server.java_address" />
			</SidebarCard>
		</template>
		Project page content
	</NormalPage>
</template>
