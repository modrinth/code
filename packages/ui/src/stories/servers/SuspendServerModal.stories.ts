import { GenericModrinthClient } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { onMounted, ref } from 'vue'

import { Button } from '../../components/base/buttons'
import SuspendServerModal from '../../components/servers/SuspendServerModal.vue'
import { provideModrinthClient } from '../../providers'

/** Creates a client whose suspend endpoint is mocked, so that the story never reaches Archon. */
function createMockClient(fail: boolean) {
	const client = new GenericModrinthClient({ userAgent: 'modrinth-storybook/1.0.0' })

	client.archon.servers_internal.suspend = async () => {
		await new Promise((resolve) => setTimeout(resolve, 600))
		if (fail) throw new Error('the server is already suspended')
	}

	return client
}

function renderStory(fail: boolean) {
	return () => ({
		components: { Button, SuspendServerModal },
		setup() {
			provideModrinthClient(createMockClient(fail))

			const modal = ref<InstanceType<typeof SuspendServerModal> | null>(null)

			onMounted(() => modal.value?.show())

			return { modal }
		},
		template: /* html */ `
			<Button @click="modal?.show()">Open modal</Button>
			<SuspendServerModal ref="modal" server-id="server-id" server-name="Survival server" />
		`,
	})
}

const meta = {
	title: 'Servers/SuspendServerModal',
	component: SuspendServerModal,
} satisfies Meta<typeof SuspendServerModal>

export default meta
type Story = StoryObj<typeof meta>

/** The modal closes and a success notification is shown. */
export const Default: Story = {
	args: { serverId: 'server-id', serverName: 'Survival server' },
	render: renderStory(false),
}

/** The modal stays open and an error notification is shown. */
export const SuspendError: Story = {
	args: { serverId: 'server-id', serverName: 'Survival server' },
	render: renderStory(true),
}
