<template>
	<div
		inert
		aria-hidden="true"
		class="relative mx-auto h-[38rem] w-full max-w-[25rem] select-none -mb-10"
	>
		<div
			class="absolute inset-x-0 top-0 h-full overflow-hidden rounded-2xl border border-solid border-surface-3 bg-surface-2 shadow-card"
		>
			<div
				class="flex items-center justify-between border-0 border-b border-solid border-surface-3 p-4"
			>
				<div class="flex items-center gap-2 text-contrast">
					<UserPlusIcon class="size-5 text-primary" />
					<span class="text-lg font-semibold">{{ formatMessage(messages.heading) }}</span>
				</div>
				<XIcon class="size-5 text-secondary" />
			</div>

			<div class="flex gap-2 border-0 border-b border-solid border-surface-3 p-4">
				<div
					class="flex h-9 min-w-0 flex-1 items-center gap-2 rounded-xl bg-surface-3 px-3 text-sm text-secondary"
				>
					<SearchIcon class="size-4 shrink-0" />
					<span class="truncate text-secondary">{{
						formatMessage(messages.searchPlaceholder)
					}}</span>
				</div>
				<Button type="colored" color="brand" class="!cursor-default" disabled>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.addButton) }}
				</Button>
			</div>

			<div class="bg-surface-1 py-3">
				<p class="m-0 pb-2 text-sm font-semibold text-primary px-4">
					{{ formatMessage(messages.friendsHeading, { count: totalFriendCount }) }}
				</p>
				<div
					v-for="(friend, index) in friends"
					:key="friend.username"
					class="relative flex h-11 items-center justify-between gap-3 px-4"
					:class="[index > 4 ? 'opacity-40' : '', friend.showPointer ? 'bg-surface-2' : '']"
				>
					<div class="flex min-w-0 items-center gap-2">
						<div class="relative flex shrink-0">
							<Avatar :src="friend.avatarUrl" size="1.5rem" class="!border-none" circle no-shadow />
							<span
								v-if="friend.presence"
								class="absolute bottom-0 right-[-1px] size-2 rounded-full border border-solid border-surface-1"
								:class="friend.presence === 'online' ? 'bg-brand' : 'bg-blue'"
							/>
						</div>
						<span class="truncate text-base font-medium text-primary">{{ friend.username }}</span>
					</div>
					<Button
						:type="friend.status === 'cancel' ? 'outlined' : 'base'"
						class="!cursor-default !font-medium !h-8"
						size="md"
						:disabled="friend.status === 'added'"
						:class="friend.status === 'added' ? '' : 'w-20'"
					>
						<CheckIcon v-if="friend.status === 'added'" aria-hidden="true" />
						{{ friendStatusLabel(friend.status) }}
					</Button>
					<div
						v-if="friend.showPointer"
						class="absolute right-[14.25rem] top-9 z-10 flex size-8 -translate-y-1/2 items-center justify-center rounded-full bg-white/10 opacity-75"
					>
						<img :src="pointerIcon" alt="" class="size-4" />
					</div>
				</div>
			</div>

			<div
				class="absolute inset-x-0 bottom-0 border-0 border-t border-solid border-surface-3 bg-surface-2 p-4"
			>
				<p class="m-0 pb-2 text-base font-semibold text-contrast">
					{{ formatMessage(messages.inviteLinkHeading) }}
				</p>
				<div
					class="flex h-8 items-center justify-between gap-2 rounded-xl bg-surface-1 px-2.5 text-sm font-medium text-primary"
				>
					<span class="truncate">https://modrinth.com/server/abc123</span>
					<ClipboardCopyIcon class="size-4 shrink-0 text-secondary" />
				</div>
			</div>
		</div>
		<div
			class="pointer-events-none absolute inset-x-0 bottom-0 h-[28rem] [background:linear-gradient(to_bottom,transparent,var(--surface-1))]"
		/>

		<div
			class="absolute left-[32%] top-[23rem] z-10 flex w-[21rem] max-w-[calc(100%-1rem)] gap-4 !overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-2 px-4 py-3 shadow-card"
		>
			<div
				class="absolute top-[16px] left-[16px] size-[20px] rounded-full bg-contrast opacity-75 blur-xl"
			/>
			<div class="relative h-max">
				<Avatar :src="geometricallyAvatar" size="2.25rem" circle no-shadow />
				<span
					class="absolute bottom-0 right-[-1px] size-3 rounded-full border-2 border-solid border-surface-2 bg-brand"
				/>
			</div>
			<div class="min-w-0 flex-1">
				<div class="flex items-start gap-1">
					<div class="min-w-0 flex-1 text-base leading-5 text-primary">
						<p class="m-0">
							<span class="font-medium text-contrast">Geometrically</span>
							{{ formatMessage(messages.invitedYouTo) }}
						</p>
						<p class="m-0 flex items-center gap-1">
							<Avatar :src="modrinthSmpIcon" size="1.25rem" no-shadow />
							<span class="font-medium text-contrast">Modrinth SMP</span>
							<span>{{ formatMessage(messages.serverSuffix) }}</span>
						</p>
					</div>
					<XIcon class="size-5 shrink-0 text-secondary" />
				</div>
				<div class="mt-2.5 flex gap-2">
					<Button type="colored" color="brand" class="!cursor-default !font-medium">
						{{ formatMessage(messages.acceptButton) }}
					</Button>
					<Button type="outlined" class="!cursor-default !font-medium">
						{{ formatMessage(messages.declineButton) }}
					</Button>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ClipboardCopyIcon,
	PlusIcon,
	SearchIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'

import borisAvatar from '#ui/assets/servers/server-list-empty/boris.png'
import fetchAvatar from '#ui/assets/servers/server-list-empty/fetch.png'
import geometricallyAvatar from '#ui/assets/servers/server-list-empty/geometrically.png'
import imb11Avatar from '#ui/assets/servers/server-list-empty/imb11.png'
import joshAvatar from '#ui/assets/servers/server-list-empty/josh.png'
import michaelAvatar from '#ui/assets/servers/server-list-empty/michael.png'
import modrinthSmpIcon from '#ui/assets/servers/server-list-empty/modrinth-smp.png'
import prospectorAvatar from '#ui/assets/servers/server-list-empty/prospector.png'
import sayaAvatar from '#ui/assets/servers/server-list-empty/saya.png'
import trumanAvatar from '#ui/assets/servers/server-list-empty/truman.png'
import Avatar from '#ui/components/base/Avatar.vue'
import Button from '#ui/components/base/buttons/Button.vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import pointerIcon from './Pointer.png'

type FriendStatus = 'added' | 'cancel' | 'invite'
type FriendPresence = 'online' | 'playing'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	heading: {
		id: 'sharing.invite-players-modal.heading',
		defaultMessage: 'Invite players',
	},
	searchPlaceholder: {
		id: 'sharing.invite-players-modal.search-placeholder',
		defaultMessage: 'Enter Modrinth username',
	},
	addButton: {
		id: 'sharing.invite-players-modal.add',
		defaultMessage: 'Add',
	},
	friendsHeading: {
		id: 'sharing.invite-players-modal.friends-heading',
		defaultMessage: 'Your friends - {count}',
	},
	addedButton: {
		id: 'sharing.invite-players-modal.added',
		defaultMessage: 'Added',
	},
	cancelButton: {
		id: 'sharing.invite-players-modal.cancel',
		defaultMessage: 'Cancel',
	},
	inviteButton: {
		id: 'sharing.invite-players-modal.invite',
		defaultMessage: 'Invite',
	},
	inviteLinkHeading: {
		id: 'sharing.invite-players-modal.invite-link-heading',
		defaultMessage: 'Or use an invite link',
	},
	invitedYouTo: {
		id: 'sharing.invite-players-toast.invited-you-to',
		defaultMessage: 'invited you to',
	},
	serverSuffix: {
		id: 'sharing.invite-players-toast.server-suffix',
		defaultMessage: 'server.',
	},
	acceptButton: {
		id: 'sharing.invite-players-toast.accept',
		defaultMessage: 'Accept',
	},
	declineButton: {
		id: 'sharing.invite-players-toast.decline',
		defaultMessage: 'Decline',
	},
})

const friends: Array<{
	username: string
	avatarUrl: string
	status: FriendStatus
	presence?: FriendPresence
	showPointer?: boolean
}> = [
	{ username: 'Josh', avatarUrl: joshAvatar, status: 'added' },
	{
		username: 'Prospector',
		avatarUrl: prospectorAvatar,
		status: 'invite',
		presence: 'online',
		showPointer: true,
	},
	{ username: 'Fetch', avatarUrl: fetchAvatar, status: 'cancel', presence: 'playing' },
	{ username: 'IMB11', avatarUrl: imb11Avatar, status: 'invite' },
	{ username: 'Truman', avatarUrl: trumanAvatar, status: 'invite' },
	{ username: 'Boris', avatarUrl: borisAvatar, status: 'invite' },
	{ username: 'Saya', avatarUrl: sayaAvatar, status: 'invite' },
	{ username: 'Michael', avatarUrl: michaelAvatar, status: 'invite' },
]

const totalFriendCount = 11

function friendStatusLabel(status: FriendStatus): string {
	if (status === 'added') return formatMessage(messages.addedButton)
	if (status === 'cancel') return formatMessage(messages.cancelButton)
	return formatMessage(messages.inviteButton)
}
</script>
