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
		docs: {
			description: {
				component: 'Role descriptions use the same instance-focused copy as the access page.',
			},
		},
	},
} satisfies Meta<typeof GrantAccessModal>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { ButtonStyled, GrantAccessModal },
		setup() {
			const modalRef = ref<InstanceType<typeof GrantAccessModal> | null>(null)
			const lastAddedUser = ref('')
			const suggestions = [
				{ id: 'fetch', username: 'Fetch' },
				{ id: 'emma', username: 'Emma' },
			]
			function handleGrant(payload: GrantServerAccessPayload) {
				lastAddedUser.value = `${payload.target} as ${payload.role}`
			}
			return { modalRef, suggestions, lastAddedUser, handleGrant }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Add a user</button>
				</ButtonStyled>
				<p v-if="lastAddedUser" class="m-0 text-sm text-secondary">Last added: {{ lastAddedUser }}</p>
				<GrantAccessModal ref="modalRef" :suggestions="suggestions" @grant="handleGrant" />
			</div>
		`,
	}),
}
