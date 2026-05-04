import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import GrantAccessModal from '../../components/servers/access/GrantAccessModal.vue'
import type { GrantServerAccessPayload } from '../../components/servers/access/types'

const meta = {
	title: 'Servers/GrantAccessModal',
	component: GrantAccessModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof GrantAccessModal>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { ButtonStyled, GrantAccessModal },
		setup() {
			const modalRef = ref<InstanceType<typeof GrantAccessModal> | null>(null)
			const lastInvite = ref('')
			const suggestions = [
				{ id: 'fetch', username: 'Fetch', email: 'fetch@example.com' },
				{ id: 'emma', username: 'Emma', email: 'emma@example.com' },
			]
			function handleGrant(payload: GrantServerAccessPayload) {
				lastInvite.value = `${payload.target} as ${payload.role}`
			}
			return { modalRef, suggestions, lastInvite, handleGrant }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Invite friend</button>
				</ButtonStyled>
				<p v-if="lastInvite" class="m-0 text-sm text-secondary">Last invite: {{ lastInvite }}</p>
				<GrantAccessModal ref="modalRef" :suggestions="suggestions" @grant="handleGrant" />
			</div>
		`,
	}),
}
