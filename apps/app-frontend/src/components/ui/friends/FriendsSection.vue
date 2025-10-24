<script setup lang="ts">
import { MoreVerticalIcon, TrashIcon, UserIcon, XIcon } from '@modrinth/assets'
import { Accordion, Avatar, ButtonStyled, OverflowMenu } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useTemplateRef } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import type { FriendWithUserData } from '@/helpers/friends.ts'

const { formatMessage } = useVIntl()

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

function createContextMenuOptions(friend: FriendWithUserData) {
	if (friend.accepted) {
		return [
			{
				name: 'view-profile',
			},
			{
				name: 'remove-friend',
				color: 'danger',
			},
		]
	} else {
		return [
			{
				name: 'view-profile',
			},
			{
				name: 'cancel-request',
			},
		]
	}
}

function openProfile(username: string) {
	openUrl('https://modrinth.com/user/' + username)
}

const friendOptions = useTemplateRef('friendOptions')
async function handleFriendOptions(args: { item: FriendWithUserData; option: string }) {
	switch (args.option) {
		case 'remove-friend':
		case 'cancel-request':
			await props.removeFriend(args.item)
			break
		case 'view-profile':
			openProfile(args.item.username)
	}
}

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
})
</script>

<template>
	<ContextMenu ref="friendOptions" @option-clicked="handleFriendOptions">
		<template #view-profile>
			<UserIcon />
			{{ formatMessage(messages.viewProfile) }}
		</template>
		<template #remove-friend> <TrashIcon /> {{ formatMessage(messages.removeFriend) }} </template>
		<template #cancel-request> <XIcon /> {{ formatMessage(messages.cancelRequest) }} </template>
	</ContextMenu>
	<Accordion
		:open-by-default="openByDefault"
		:force-open="isSearching"
		:button-class="
			'pl-4 pr-3 flex w-full items-center bg-transparent border-0 p-0' +
			(isSearching
				? ''
				: ' cursor-pointer hover:brightness-[--hover-brightness] active:scale-[0.98] transition-all')
		"
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
					class="group grid items-center grid-cols-[auto_1fr_auto] gap-2 hover:bg-button-bg transition-colors rounded-full ml-4 mr-1"
					@contextmenu.prevent.stop="
						(event) => friendOptions?.showMenu(event, friend, createContextMenuOptions(friend))
					"
				>
					<div class="relative">
						<Avatar
							:src="friend.avatar"
							:class="{ grayscale: !friend.online && friend.accepted }"
							class="w-12 h-12 rounded-full"
							size="32px"
							circle
						/>
						<span
							v-if="friend.online"
							aria-hidden="true"
							class="bottom-[2px] right-[-2px] absolute w-3 h-3 bg-brand border-2 border-black border-solid rounded-full"
						/>
					</div>
					<div class="flex flex-col">
						<span
							class="text-sm m-0"
							:class="friend.online || !friend.accepted ? 'text-contrast' : 'text-primary'"
						>
							{{ friend.username }}
						</span>
						<span v-if="!friend.accepted" class="m-0 text-xs">
							{{ formatMessage(messages.friendRequestSent) }}
						</span>
						<span v-else-if="friend.status" class="m-0 text-xs">{{ friend.status }}</span>
					</div>
					<ButtonStyled v-if="friend.accepted" circular type="transparent">
						<OverflowMenu
							class="opacity-0 group-hover:opacity-100 transition-opacity"
							:options="[
								{
									id: 'view-profile',
									action: () => openProfile(friend.username),
								},
								{
									id: 'remove-friend',
									action: () => removeFriend(friend),
									color: 'red',
								},
							]"
						>
							<MoreVerticalIcon />
							<template #view-profile>
								<UserIcon />
								{{ formatMessage(messages.viewProfile) }}
							</template>
							<template #remove-friend>
								<TrashIcon />
								{{ formatMessage(messages.removeFriend) }}
							</template>
						</OverflowMenu>
					</ButtonStyled>
					<ButtonStyled v-else type="transparent" circular>
						<button v-tooltip="formatMessage(messages.cancelRequest)" @click="removeFriend(friend)">
							<XIcon />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</Accordion>
</template>
