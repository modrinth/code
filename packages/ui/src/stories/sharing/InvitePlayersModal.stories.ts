import type { Labrinth } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import InvitePlayersModal from '../../components/sharing/invite-players-modal/index.vue'
import type {
	InvitePlayersInvitePayload,
	InvitePlayersUser,
	InvitePlayersUserStatus,
} from '../../components/sharing/invite-players-modal/types'

const apiUsers: Labrinth.Users.v3.SearchUser[] = [
	{
		id: 'geometrically',
		username: 'Geometrically',
		avatar_url:
			'https://cdn.modrinth.com/user/u6dRKJwZ/7ba3bdb11590a64843e9d2ab83ef85eaab42ec8e.png',
	},
	{
		id: 'prospector',
		username: 'Prospector',
		avatar_url:
			'https://cdn.modrinth.com/user/PHyAPGui/30a3a3f53866531831db4aa006794e6bbcfc4121.png',
	},
	{
		id: 'fetch',
		username: 'Fetch',
		avatar_url:
			'https://cdn.modrinth.com/user/yol4bNw3/ee2c7a7580ed475cfe3cfe8cc92df45ce33031e0.png',
	},
	{
		id: 'imb11',
		username: 'IMB11',
		avatar_url: null,
	},
	{
		id: 'josh',
		username: 'Josh',
		avatar_url: null,
	},
	{
		id: 'emma',
		username: 'Emma',
		avatar_url: null,
	},
]

const friendStatuses: Record<string, InvitePlayersUserStatus> = {
	geometrically: 'added',
	fetch: 'pending',
}

const meta = {
	title: 'Sharing/InvitePlayersModal',
	component: InvitePlayersModal,
	parameters: {
		layout: 'centered',
		docs: {
			description: {
				component:
					'Invite players modal for app instance sharing and server player invites. Callers provide the user search proxy and own invite/cancel persistence.',
			},
		},
	},
} satisfies Meta<typeof InvitePlayersModal>

export default meta
type Story = StoryObj<typeof meta>

function toInviteUser(
	user: Labrinth.Users.v3.SearchUser,
	status: InvitePlayersUserStatus = 'available',
): InvitePlayersUser {
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url,
		status,
		online: user.id === 'prospector',
	}
}

function createFriends() {
	return apiUsers
		.slice(0, 4)
		.map((user) => toInviteUser(user, friendStatuses[user.id] ?? 'available'))
}

function createSearchUsers() {
	return apiUsers.map((user) => ({
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url,
	}))
}

function createRender(args: Record<string, unknown>) {
	return {
		components: { ButtonStyled, InvitePlayersModal },
		setup() {
			const modalRef = ref<InstanceType<typeof InvitePlayersModal> | null>(null)
			const friends = ref<InvitePlayersUser[]>(createFriends())
			const searchUsers = createSearchUsers()
			const lastAction = ref('')
			const manuallyAddedFriendIds = ref(new Set<string>())

			async function searchInviteUsers(query: string) {
				await new Promise((resolve) => setTimeout(resolve, 250))
				const normalizedQuery = query.trim().toLowerCase()
				const friendKeys = new Set(
					friends.value.flatMap((friend) => [
						friend.id.toLowerCase(),
						friend.username.toLowerCase(),
					]),
				)

				return searchUsers.filter(
					(user) =>
						user.username.toLowerCase().startsWith(normalizedQuery) &&
						!friendKeys.has(user.id.toLowerCase()) &&
						!friendKeys.has(user.username.toLowerCase()),
				)
			}

			function handleInvite(payload: InvitePlayersInvitePayload) {
				const existingFriend = friends.value.find((friend) => friend.id === payload.user.id)

				if (payload.source === 'search') {
					if (existingFriend) {
						existingFriend.status = 'requested'
					} else {
						friends.value = [{ ...payload.user, status: 'requested' }, ...friends.value]
					}

					manuallyAddedFriendIds.value = new Set([...manuallyAddedFriendIds.value, payload.user.id])
					lastAction.value = `Sent friend request to ${payload.user.username}`
					return
				}

				if (existingFriend) {
					existingFriend.status = 'pending'
				}

				lastAction.value = `Invited ${payload.user.username} from ${payload.source}`
			}

			function handleCancel(user: InvitePlayersUser) {
				const existingFriend = friends.value.find((friend) => friend.id === user.id)
				if (manuallyAddedFriendIds.value.has(user.id)) {
					friends.value = friends.value.filter((friend) => friend.id !== user.id)
					const nextManuallyAddedFriendIds = new Set(manuallyAddedFriendIds.value)
					nextManuallyAddedFriendIds.delete(user.id)
					manuallyAddedFriendIds.value = nextManuallyAddedFriendIds
					lastAction.value = `Cancelled friend request for ${user.username}`
					return
				}

				if (existingFriend) existingFriend.status = 'available'
				lastAction.value = `Cancelled invite for ${user.username}`
			}

			return {
				args,
				friends,
				handleCancel,
				handleInvite,
				lastAction,
				modalRef,
				searchInviteUsers,
			}
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="brand">
					<button @click="modalRef?.show($event)">Open modal</button>
				</ButtonStyled>
				<p v-if="lastAction" class="m-0 text-sm text-secondary">{{ lastAction }}</p>
				<InvitePlayersModal
					ref="modalRef"
					v-bind="args"
					:friends="friends"
					:search-users="searchInviteUsers"
					@invite="handleInvite"
					@cancel="handleCancel"
				/>
			</div>
		`,
	}
}

export const ShareInstance: Story = {
	args: {
		header: 'Share instance',
		link: 'https://modrinth.com/instance/abc123',
	},
	render: (args) => createRender(args),
}

export const FriendsOnly: Story = {
	args: {
		header: 'Invite players',
	},
	render: (args) => createRender(args),
}
