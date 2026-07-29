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
					'Role descriptions use the same instance-focused copy as the access page. The username field searches users asynchronously before showing the empty state.',
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
			async function searchUsers(query: string) {
				await new Promise((resolve) => setTimeout(resolve, 250))
				const normalizedQuery = query.trim().toLowerCase()
				return users.filter((user) => user.username.toLowerCase().startsWith(normalizedQuery))
			}
			function handleGrant(payload: GrantServerAccessPayload) {
				lastAddedUser.value = `${payload.target} as ${payload.role}${
					payload.addAsFriend ? ' with friend request' : ''
				}`
			}
			return { modalRef, searchUsers, lastAddedUser, handleGrant }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Add user</button>
				</ButtonStyled>
				<p v-if="lastAddedUser" class="m-0 text-sm text-secondary">Last added: {{ lastAddedUser }}</p>
				<GrantAccessModal ref="modalRef" :search-users="searchUsers" @grant="handleGrant" />
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
			async function searchUsers(query: string) {
				await new Promise((resolve) => setTimeout(resolve, 250))
				const normalizedQuery = query.trim().toLowerCase()
				return users.filter((user) => user.username.toLowerCase().startsWith(normalizedQuery))
			}
			return { modalRef, members, searchUsers }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Add existing user</button>
				</ButtonStyled>
				<GrantAccessModal ref="modalRef" :members="members" :search-users="searchUsers" />
			</div>
		`,
	}),
}
