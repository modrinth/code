<script setup lang="ts">
import {
	ClipboardCopyIcon,
	ExternalIcon,
	LinkIcon,
	MailIcon,
	MoreVerticalIcon,
	UserPlusIcon,
	UsersIcon,
} from '@modrinth/assets'
import {
	Avatar,
	type ButtonMenuOption,
	injectModrinthClient,
	Table,
	type TableColumn,
	TeleportOverflowMenu,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { useQueries } from '@tanstack/vue-query'
import { computed } from 'vue'

import { useAuthState } from '~/composables/auth'
import { creatorProjectStatsQueryOptions } from '~/composables/creator-project-stats'
import type { SharedInstanceModerationUser } from '~/composables/queries/shared-instance'
import { useCopyText } from '~/composables/shared-instance-moderation'

type ColumnKey = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

const props = defineProps<{
	members: SharedInstanceModerationUser[]
}>()

const client = injectModrinthClient()
const copyText = useCopyText()
const auth = useAuthState()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const columns: TableColumn<ColumnKey>[] = [
	{ key: 'username', label: 'Username' },
	{
		key: 'lastPlayed',
		label: 'Last played',
		width: '10rem',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'joined',
		label: 'Joined',
		width: '10rem',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'method',
		label: 'Join method',
		width: '12rem',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{ key: 'actions', label: 'Actions', width: '6rem', align: 'right' },
]

const isAdmin = computed(() => auth.value.user?.role === 'admin')

const accounts = useQueries({
	queries: computed(() =>
		isAdmin.value
			? props.members.map((member) =>
					creatorProjectStatsQueryOptions.memberAccount(member.id, client),
				)
			: [],
	),
})

function memberEmail(index: number) {
	return accounts.value[index]?.data?.email
}

function memberMethod(member: SharedInstanceModerationUser) {
	if (member.join_type === 'owner') return { icon: UsersIcon, label: 'Owner' }
	if (member.join_type === 'link') return { icon: LinkIcon, label: 'Share link' }
	return { icon: UserPlusIcon, label: 'Direct invite' }
}

function memberOptions(member: SharedInstanceModerationUser, index: number): ButtonMenuOption[] {
	const email = memberEmail(index)
	return [
		{
			id: 'open',
			type: 'link',
			label: 'Open in new tab',
			icon: ExternalIcon,
			href: `/user/${encodeURIComponent(member.id)}`,
			target: '_blank',
			rel: 'noopener noreferrer',
		},
		{
			id: 'copy-email',
			label: 'Copy email',
			icon: MailIcon,
			shown: isAdmin.value,
			disabled: !email,
			action: () => {
				if (email) void copyText(email, 'Email copied')
			},
		},
		{
			id: 'copy-id',
			label: 'Copy ID',
			icon: ClipboardCopyIcon,
			action: () => copyText(member.id, 'User ID copied'),
		},
	]
}
</script>

<template>
	<div class="overflow-y-auto p-6">
		<Table :columns="columns" :data="members" row-key="id" table-min-width="42rem">
			<template #cell-username="{ row }">
				<NuxtLink
					:to="`/user/${row.id}`"
					target="_blank"
					class="inline-flex min-w-0 max-w-full items-center gap-2 text-primary hover:underline"
				>
					<Avatar
						:src="row.avatar_url"
						:alt="`${row.username}'s avatar`"
						:tint-by="row.id"
						size="24px"
						circle
						no-shadow
					/>
					<span class="min-w-0 truncate font-medium">{{ row.username }}</span>
				</NuxtLink>
			</template>
			<template #cell-lastPlayed="{ row }">
				<span v-if="row.last_played" v-tooltip="formatDateTime(row.last_played)">
					{{ formatRelativeTime(row.last_played) }}
				</span>
				<span v-else>Never</span>
			</template>
			<template #cell-joined="{ row }">
				<span v-if="row.joined_at" v-tooltip="formatDateTime(row.joined_at)">
					{{ formatRelativeTime(row.joined_at) }}
				</span>
				<span v-else>Unknown</span>
			</template>
			<template #cell-method="{ row }">
				<span class="inline-flex min-w-0 items-center gap-2">
					<component :is="memberMethod(row).icon" class="size-5 shrink-0" aria-hidden="true" />
					<span class="min-w-0 truncate">{{ memberMethod(row).label }}</span>
				</span>
			</template>
			<template #cell-actions="{ row, index }">
				<div class="flex justify-end">
					<TeleportOverflowMenu
						type="quiet"
						:label="`More options for ${row.username}`"
						:options="memberOptions(row, index)"
					>
						<MoreVerticalIcon />
					</TeleportOverflowMenu>
				</div>
			</template>
		</Table>
	</div>
</template>
