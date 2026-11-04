import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import SuspendServerModal from '../../components/servers/SuspendServerModal.vue'

const meta = {
	title: 'Servers/SuspendServerModal',
	component: SuspendServerModal,
} satisfies Meta<typeof SuspendServerModal>

export default meta

export const Default: StoryObj<typeof meta> = {
	args: {
		serverId: 'server-id',
		serverName: 'Survival server',
	},
	render: (args) => ({
		components: { Button, SuspendServerModal },
		setup() {
			const modal = ref<InstanceType<typeof SuspendServerModal> | null>(null)
			return { args, modal }
		},
		template: `
			<Button @click="modal?.show()">Open modal</Button>
			<SuspendServerModal ref="modal" v-bind="args" />
		`,
	}),
}
