import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, RefreshCwIcon } from '@modrinth/assets'
import type { GameVersionTag } from '@modrinth/utils'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button, IconButton } from '../../components/base/buttons'
import VersionFilterControl from '../../components/version/VersionFilterControl.vue'

const gameVersions: GameVersionTag[] = [
	{ version: '1.21.4', version_type: 'release', date: '2024-12-03', major: true },
	{ version: '1.21.1', version_type: 'release', date: '2024-08-08', major: true },
	{ version: '24w14a', version_type: 'snapshot', date: '2024-04-03', major: false },
	{ version: '1.20.1', version_type: 'release', date: '2023-06-12', major: true },
]

function version(
	id: string,
	gameVersions: string[],
	loaders: string[],
	versionType: Labrinth.Versions.v3.Version['version_type'],
	environment: Labrinth.Projects.v3.Environment,
) {
	return {
		id,
		game_versions: gameVersions,
		loaders,
		version_type: versionType,
		environment,
	} as Labrinth.Versions.v3.Version
}

const versions = [
	version('a', ['1.21.4', '1.21.1'], ['fabric', 'quilt'], 'release', 'client_only'),
	version('b', ['24w14a'], ['fabric'], 'alpha', 'client_only'),
	version('c', ['1.20.1'], ['forge', 'neoforge'], 'beta', 'client_and_server'),
]

const meta = {
	title: 'Version/VersionFilterControl',
	component: VersionFilterControl,
	args: {
		versions,
		gameVersions,
		searchPlaceholder: 'Search 3 versions...',
		page: 1,
		pageCount: 0,
	},
	render: (args) => ({
		components: { VersionFilterControl },
		setup() {
			const search = ref('')
			return { args, search }
		},
		template: /* html */ `
			<VersionFilterControl v-bind="args" v-model:search="search" />
		`,
	}),
} satisfies Meta<typeof VersionFilterControl>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithPagination: Story = {
	args: {
		pageCount: 5,
	},
	render: (args) => ({
		components: { VersionFilterControl },
		setup() {
			const search = ref('')
			const page = ref(args.page ?? 1)
			return { args, search, page }
		},
		template: /* html */ `
			<VersionFilterControl
				v-bind="args"
				v-model:search="search"
				:page="page"
				@switch-page="(next) => (page = next)"
			/>
		`,
	}),
}

export const WithActions: Story = {
	render: (args) => ({
		components: { VersionFilterControl, Button, IconButton, PlusIcon, RefreshCwIcon },
		setup() {
			const search = ref('')
			return { args, search }
		},
		template: /* html */ `
			<VersionFilterControl v-bind="args" v-model:search="search">
				<template #actions>
					<IconButton type="quiet" size="lg" label="Refresh">
						<RefreshCwIcon aria-hidden="true" />
					</IconButton>
					<Button color="green" type="colored" size="lg">
						<PlusIcon /> Create version
					</Button>
				</template>
			</VersionFilterControl>
		`,
	}),
}
