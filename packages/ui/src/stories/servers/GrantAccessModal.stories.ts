import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import GrantAccessModal from '../../components/servers/access/GrantAccessModal.vue'
import type {
	GrantServerAccessPayload,
	ServerAccessMember,
} from '../../components/servers/access/types'

const meta = {
	title: 'Servers/GrantAccessModal',
	component: GrantAccessModal,
	parameters: {
		layout: 'centered',
		docs: {
			description: {
				component:
					'Role descriptions use the same instance-focused copy as the access page. The username field resolves users asynchronously before showing the empty state.',
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
			const users = [
				{ id: 'fetch', username: 'Fetch' },
				{ id: 'emma', username: 'Emma' },
			]
			async function resolveUser(target: string) {
				await new Promise((resolve) => setTimeout(resolve, 250))
				const normalizedTarget = target.trim().toLowerCase()
				return users.find((user) => user.username.toLowerCase() === normalizedTarget) ?? null
			}
			function handleGrant(payload: GrantServerAccessPayload) {
				lastAddedUser.value = `${payload.target} as ${payload.role}${
					payload.addAsFriend ? ' with friend request' : ''
				}`
			}
			return { modalRef, resolveUser, lastAddedUser, handleGrant }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Add user</button>
				</ButtonStyled>
				<p v-if="lastAddedUser" class="m-0 text-sm text-secondary">Last added: {{ lastAddedUser }}</p>
				<GrantAccessModal ref="modalRef" :resolve-user="resolveUser" @grant="handleGrant" />
			</div>
		`,
	}),
}

export const ExistingMember: Story = {
	render: () => ({
		components: { ButtonStyled, GrantAccessModal },
		setup() {
			const modalRef = ref<InstanceType<typeof GrantAccessModal> | null>(null)
			const users = [
				{ id: 'josh11', username: 'josh11' },
				{ id: 'emma', username: 'Emma' },
			]
			const members: ServerAccessMember[] = [
				{
					id: 'story-josh11',
					user: {
						id: 'josh11',
						username: 'josh11',
					},
					role: 'editor',
					joinedAt: new Date().toISOString(),
				},
			]
			async function resolveUser(target: string) {
				await new Promise((resolve) => setTimeout(resolve, 250))
				const normalizedTarget = target.trim().toLowerCase()
				return users.find((user) => user.username.toLowerCase() === normalizedTarget) ?? null
			}
			return { modalRef, members, resolveUser }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Add existing user</button>
				</ButtonStyled>
				<GrantAccessModal ref="modalRef" :members="members" :resolve-user="resolveUser" />
			</div>
		`,
	}),
}
