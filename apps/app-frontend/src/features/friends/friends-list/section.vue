<script setup lang="ts">
import { MoreVerticalIcon, TrashIcon, UserIcon, XIcon } from '@modrinth/assets'
import type { ButtonMenuOption } from '@modrinth/ui'
import {
	Accordion,
	ContextMenu,
	defineMessages,
	IconButton,
	TeleportOverflowMenu,
	UserAvatar,
	useVIntl,
} from '@modrinth/ui'
import { useTemplateRef } from 'vue'
import { useRouter } from 'vue-router'

import type { FriendWithUserData } from '@/helpers/friends.ts'

const { formatMessage } = useVIntl()
const router = useRouter()

const props = withDefaults(
	defineProps<{
		friends: FriendWithUserData[]
		heading: string
		removeFriend: (friend: FriendWithUserData) => Promise<void>
		isSearching?: boolean
		openByDefault?: boolean
	}>(),
	{
		isSearching: false,
		openByDefault: false,
	},
)

const emit = defineEmits<{
	onOpen: []
	onClose: []
}>()

function createContextMenuOptions(friend: FriendWithUserData): ButtonMenuOption[] {
	return [
		{
			id: 'view-profile',
			label: formatMessage(messages.viewProfile),
			icon: UserIcon,
			action: () => openProfile(friend.username),
		},
		friend.accepted
			? {
					id: 'remove-friend',
					label: formatMessage(messages.removeFriend),
					icon: TrashIcon,
					tone: 'red',
					hoverFilledOnly: true,
					action: () => void props.removeFriend(friend),
				}
			: {
					id: 'cancel-request',
					label: formatMessage(messages.cancelRequest),
					icon: XIcon,
					action: () => void props.removeFriend(friend),
				},
	]
}

function openProfile(username: string) {
	void router.push(`/user/${encodeURIComponent(username)}`)
}

const friendOptions = useTemplateRef('friendOptions')

const messages = defineMessages({
	removeFriend: {
		id: 'friends.friend.remove-friend',
		defaultMessage: 'Remove friend',
	},
	heading: {
		id: 'friends.section.heading',
		defaultMessage: '{title} - {count}',
	},
	friendRequestSent: {
		id: 'friends.friend.request-sent',
		defaultMessage: 'Friend request sent',
	},
	cancelRequest: {
		id: 'friends.friend.cancel-request',
		defaultMessage: 'Cancel request',
	},
	viewProfile: {
		id: 'friends.friend.view-profile',
		defaultMessage: 'View profile',
	},
	friendActionsLabel: {
		id: 'friends.friend.actions.label',
		defaultMessage: 'Friend actions',
	},
})
</script>

<template>
	<ContextMenu ref="friendOptions" :label="formatMessage(messages.friendActionsLabel)" />
	<Accordion
		:open-by-default="openByDefault"
		:force-open="isSearching"
		:button-class="
			'flex w-full items-center bg-transparent border-0 p-0' +
			(isSearching
				? ''
				: ' cursor-pointer hover:brightness-[--hover-brightness] active:scale-[0.98] transition-all')
		"
		@on-open="emit('onOpen')"
		@on-close="emit('onClose')"
	>
		<template #title>
			<h3 class="text-base text-primary font-medium m-0">
				{{ formatMessage(messages.heading, { title: heading, count: friends.length }) }}
			</h3>
		</template>
		<template #default>
			<div class="pt-3 flex flex-col gap-1">
				<div
					v-for="friend in friends"
					:key="friend.username"
					class="group grid items-center grid-cols-[1fr_auto] gap-2 hover:bg-button-bg transition-colors rounded-full mr-1 select-none"
					@contextmenu.prevent.stop="
						(event) => friendOptions?.open(event, createContextMenuOptions(friend))
					"
				>
					<RouterLink
						:to="`/user/${encodeURIComponent(friend.username)}`"
						class="grid min-w-0 grid-cols-[auto_1fr] items-center gap-2 text-inherit no-underline group no-click-animation"
					>
						<UserAvatar
							:src="friend.avatar"
							size="32px"
							:badge="friend.online"
							:grayscale="!friend.online && friend.accepted"
						/>
						<div class="flex flex-col">
							<span
								class="text-sm m-0 group-hover:underline"
								:class="friend.online || !friend.accepted ? 'text-contrast' : 'text-primary'"
							>
								{{ friend.username }}
							</span>
							<span v-if="!friend.accepted" class="m-0 text-xs">
								{{ formatMessage(messages.friendRequestSent) }}
							</span>
							<span v-else-if="friend.status" class="m-0 text-xs">{{ friend.status }}</span>
						</div>
					</RouterLink>
					<TeleportOverflowMenu
						v-if="friend.accepted"
						type="quiet"
						label="More options"
						class="opacity-0 group-hover:opacity-100 transition-opacity"
						:options="[
							{
								id: 'remove-friend',
								label: formatMessage(messages.removeFriend),
								action: () => removeFriend(friend),
								tone: 'red',
							},
						]"
					>
						<MoreVerticalIcon />
						<template #remove-friend>
							<TrashIcon />
							{{ formatMessage(messages.removeFriend) }}
						</template>
					</TeleportOverflowMenu>
					<IconButton
						v-else
						v-tooltip="formatMessage(messages.cancelRequest)"
						type="quiet"
						:label="formatMessage(messages.cancelRequest)"
						@click="removeFriend(friend)"
					>
						<XIcon />
					</IconButton>
				</div>
			</div>
		</template>
	</Accordion>
</template>
