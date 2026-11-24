import { type Archon, GenericModrinthClient } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import PageHeaderMetadata from '../../components/base/page-header/metadata/index.vue'
import ServerHeaderMetadataItems from '../../components/servers/server-header/ServerHeaderMetadataItems.vue'
import { provideModrinthClient } from '../../providers'

function createMockClient(serverId: string) {
	const client = new GenericModrinthClient({ userAgent: 'modrinth-storybook/1.0.0' })

	client.archon.servers_v0.get = async () =>
		({
			server_id: serverId,
			name: 'Survival server',
			net: { domain: 'survival' },
			upstream: null,
		}) as unknown as Archon.Servers.v0.Server

	return client
}

const meta = {
	title: 'Servers/ServerHeaderMetadataItems',
	component: ServerHeaderMetadataItems,
	render: (args) => ({
		components: { PageHeaderMetadata, ServerHeaderMetadataItems },
		setup() {
			provideModrinthClient(createMockClient(args.serverId))
			return { args }
		},
		template: /* html */ `
			<PageHeaderMetadata>
				<ServerHeaderMetadataItems v-bind="args" />
			</PageHeaderMetadata>
		`,
	}),
} satisfies Meta<typeof ServerHeaderMetadataItems>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: { serverId: 'story-server', loader: 'fabric', gameVersion: '1.21.4' },
}

export const WithUptime: Story = {
	args: {
		serverId: 'story-server-uptime',
		loader: 'paper',
		gameVersion: '1.21.4',
		uptime: '3h 12m',
	},
}
