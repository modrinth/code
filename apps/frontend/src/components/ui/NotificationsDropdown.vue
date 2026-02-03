<template>
	<ButtonStyled type="transparent">
		<OverflowMenu
			ref="notificationsOverflow"
			:dropdown-id="effectiveDropdownId"
			class="btn-dropdown-animation relative flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
			:options="[]"
			@dblclick="handleViewAllNotifications"
		>
			<div class="relative flex h-5 flex-shrink-0 items-center justify-center">
				<BellIcon aria-hidden="true" class="h-5 w-5" style="transform: none" />
				<div
					v-if="unreadCount > 0"
					class="absolute -right-2 -top-2 flex h-4 w-4 items-center justify-center rounded-full bg-brand text-[10px] font-bold text-brand-inverted"
				>
					{{ unreadCount }}
				</div>
			</div>
			<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
			<template #menu-header>
				<div class="notifications-dropdown flex min-w-[300px] flex-col gap-2 p-2">
					<div class="flex items-center justify-between gap-2 rounded-lg">
						<button class="iconified-button" @click="handleViewAllNotifications">
							<BellIcon aria-hidden="true" />
							{{ formatMessage(messages.viewAllNotifications) }}
						</button>
						<button v-if="unreadCount > 0" class="iconified-button" @click="handleMarkAllAsRead">
							<CheckCheckIcon />
							{{ formatMessage(messages.markAllAsRead) }}
						</button>
					</div>
					<div class="border-t border-divider"></div>
					<div
						v-if="recentNotifications.length === 0"
						class="flex items-center justify-center rounded-lg bg-transparent py-4 text-secondary"
					>
						{{ formatMessage(messages.noUnreadNotifications) }}
					</div>
					<div
						v-for="notif in recentNotifications"
						:key="notif.id"
						class="universal-card recessed group !mb-0 flex items-center gap-2 !p-4 transition-colors hover:bg-button-bg"
					>
						<DoubleIcon class="flex-shrink-0">
							<template #primary>
								<NuxtLink
									v-if="notif.extra_data?.project"
									:to="`/project/${notif.extra_data.project.slug}`"
									tabindex="-1"
									@click="notificationsOverflow?.close()"
								>
									<Avatar size="xs" :src="notif.extra_data.project.icon_url" aria-hidden="true" />
								</NuxtLink>
								<NuxtLink
									v-else-if="notif.extra_data?.organization"
									:to="`/organization/${notif.extra_data.organization.slug}`"
									tabindex="-1"
									@click="notificationsOverflow?.close()"
								>
									<Avatar
										size="xs"
										:src="notif.extra_data.organization.icon_url"
										aria-hidden="true"
									/>
								</NuxtLink>
								<NuxtLink
									v-else-if="notif.extra_data?.user"
									:to="`/user/${notif.extra_data.user.username}`"
									tabindex="-1"
									@click="notificationsOverflow?.close()"
								>
									<Avatar size="xs" :src="notif.extra_data.user.avatar_url" aria-hidden="true" />
								</NuxtLink>
								<Avatar v-else size="xs" aria-hidden="true" />
							</template>
							<template #secondary>
								<ScaleIcon
									v-if="
										notif.body?.type === 'moderator_message' || notif.body?.type === 'status_change'
									"
									class="text-contrast"
								/>
								<UserPlusIcon
									v-else-if="notif.body?.type === 'team_invite' && notif.extra_data?.project"
									class="text-contrast"
								/>
								<UserPlusIcon
									v-else-if="
										notif.body?.type === 'organization_invite' && notif.extra_data?.organization
									"
									class="text-contrast"
								/>
								<VersionIcon
									v-else-if="
										notif.body?.type === 'project_update' &&
										notif.extra_data?.project &&
										notif.extra_data?.version
									"
									class="text-contrast"
								/>
								<BellIcon v-else class="text-contrast" />
							</template>
						</DoubleIcon>
						<div class="min-w-0 flex-1 pr-2">
							<NuxtLink
								:to="notif.link"
								class="font-semibold text-contrast"
								@click="notificationsOverflow?.close()"
							>
								{{ notif.title }}
							</NuxtLink>
							<div class="mt-1 flex items-center gap-1 text-sm text-secondary">
								<CalendarIcon aria-hidden="true" />
								{{ formatRelativeTime(notif.created) }}
							</div>
						</div>
						<div class="flex gap-2">
							<button
								v-if="
									(notif.body?.type === 'team_invite' ||
										notif.body?.type === 'organization_invite') &&
									!notif.read
								"
								class="iconified-button square-button brand-button [&>svg]:mr-0"
								@click.stop="handleAcceptInvite(notif)"
							>
								<CheckIcon />
							</button>
							<button
								v-if="
									(notif.body?.type === 'team_invite' ||
										notif.body?.type === 'organization_invite') &&
									!notif.read
								"
								class="iconified-button square-button danger-button [&>svg]:mr-0"
								@click.stop="handleDeclineInvite(notif)"
							>
								<XIcon />
							</button>
							<button
								v-else-if="!notif.read"
								class="iconified-button square-button [&>svg]:mr-0"
								@click.stop="handleMarkAsRead(notif)"
							>
								<CheckIcon />
							</button>
						</div>
					</div>
				</div>
			</template>
		</OverflowMenu>
	</ButtonStyled>
</template>

<script setup>
import {
	BellIcon,
	CalendarIcon,
	CheckCheckIcon,
	CheckIcon,
	DropdownIcon,
	ScaleIcon,
	UserPlusIcon,
	VersionIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	defineMessages,
	DoubleIcon,
	OverflowMenu,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import {
	fetchExtraNotificationData,
	groupNotifications,
	markAsRead,
} from '~/helpers/platform-notifications'
import { acceptTeamInvite, removeSelfFromTeam } from '~/helpers/teams'

const props = defineProps({
	dropdownId: {
		type: String,
		default: '',
	},
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const router = useNativeRouter()
const auth = await useAuth()

const effectiveDropdownId = computed(() => props.dropdownId || `notifications-dropdown-${useId()}`)

const { data: notificationsData, refresh: refreshNotifications } = await useAsyncData(
	'notifications-dropdown',
	async () => {
		if (!auth.value.user) return null

		const notifs = await useBaseFetch(`user/${auth.value.user.id}/notifications`)
		return await fetchExtraNotificationData(notifs)
	},
	{
		watch: [auth],
	},
)

const unreadCount = computed(() => {
	if (!notificationsData.value) return 0
	const grouped = groupNotifications(
		notificationsData.value.filter((n) => !n.read),
		false,
	)
	return grouped.length
})

const recentNotifications = computed(() => {
	if (!notificationsData.value) return []
	const unread = notificationsData.value.filter((n) => !n.read)
	return groupNotifications(unread.slice(0, 10), false)
})

const notificationsOverflow = ref(null)

const messages = defineMessages({
	viewAllNotifications: {
		id: 'layout.notifications.view-all',
		defaultMessage: 'View all',
	},
	markAllAsRead: {
		id: 'layout.notifications.mark-all-read',
		defaultMessage: 'Mark all as read',
	},
	noUnreadNotifications: {
		id: 'layout.notifications.no-unread',
		defaultMessage: 'No unread notifications',
	},
})

async function handleAcceptInvite(notif) {
	try {
		await acceptTeamInvite(notif.body.team_id)
		await markAsRead([notif.id])
		await refreshNotifications()
	} catch (err) {
		console.error('Error accepting invite:', err)
	}
}

async function handleDeclineInvite(notif) {
	try {
		await removeSelfFromTeam(notif.body.team_id)
		await markAsRead([notif.id])
		await refreshNotifications()
	} catch (err) {
		console.error('Error declining invite:', err)
	}
}

async function handleMarkAsRead(notif) {
	try {
		await markAsRead([notif.id])
		await refreshNotifications()
	} catch (err) {
		console.error('Error marking as read:', err)
	}
}

function handleViewAllNotifications() {
	notificationsOverflow.value?.close()
	router.push('/dashboard/notifications')
}

async function handleMarkAllAsRead() {
	try {
		const ids = notificationsData.value?.map((n) => n.id) || []
		await markAsRead(ids)
		await refreshNotifications()
		notificationsOverflow.value?.close()
	} catch (err) {
		console.error('Error marking all as read:', err)
	}
}
</script>

