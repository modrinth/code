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
				<div class="notifications-dropdown flex flex-col p-2">
					<div class="flex items-center justify-between gap-2 rounded-lg">
						<button class="iconified-button" @click="handleViewAllNotifications">
							<BellIcon aria-hidden="true" />
							{{ formatMessage(messages.viewAllNotifications) }}
						</button>
						<button class="iconified-button" @click="handleViewHistory">
							<HistoryIcon />
							{{ formatMessage(messages.viewHistory) }}
						</button>
						<button
							v-if="unreadCount > 0"
							class="iconified-button danger-button"
							@click="handleMarkAllAsRead"
						>
							<CheckCheckIcon />
							{{ formatMessage(messages.markAllAsRead) }}
						</button>
					</div>
					<div class="mt-2 border-t border-divider"></div>
					<div
						v-if="recentNotifications.length === 0"
						class="flex items-center justify-center rounded-lg bg-transparent py-4 text-secondary"
					>
						{{ formatMessage(messages.noUnreadNotifications) }}
					</div>
					<div class="mt-2">
						<SmartClickable v-for="notif in recentNotifications" :key="notif.id" class="w-full">
							<template #clickable>
								<NuxtLink
									:to="notif.link"
									class="no-outline no-click-animation rounded-xl"
									@click="handleNotificationClick(notif)"
								></NuxtLink>
							</template>
							<div
								class="universal-card recessed smart-clickable:highlight-on-hover group !mb-0 flex gap-2 !p-4 hover:bg-button-bg"
							>
								<DoubleIcon class="flex-shrink-0">
									<template #primary>
										<NuxtLink
											v-if="notif.extra_data?.project"
											:to="`/project/${notif.extra_data.project.slug}`"
											tabindex="-1"
											class="smart-clickable:allow-pointer-events"
											@click.stop
										>
											<Avatar
												size="xs"
												:src="notif.extra_data.project.icon_url"
												aria-hidden="true"
											/>
										</NuxtLink>
										<NuxtLink
											v-else-if="notif.extra_data?.organization"
											:to="`/organization/${notif.extra_data.organization.slug}`"
											tabindex="-1"
											class="smart-clickable:allow-pointer-events"
											@click.stop
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
											class="smart-clickable:allow-pointer-events"
											@click.stop
										>
											<Avatar
												size="xs"
												:src="notif.extra_data.user.avatar_url"
												aria-hidden="true"
											/>
										</NuxtLink>
										<Avatar v-else size="xs" aria-hidden="true" />
									</template>
									<template #secondary>
										<ScaleIcon
											v-if="
												notif.body?.type === 'moderator_message' ||
												notif.body?.type === 'status_change'
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
								<div class="w-0 min-w-0 flex-1 pr-2">
									<div class="break-words font-semibold text-contrast">{{ notif.title }}</div>
									<div class="mt-1 flex items-center gap-1 text-sm text-secondary">
										<CalendarIcon aria-hidden="true" />
										{{ formatRelativeTime(notif.created) }}
									</div>
								</div>
								<div class="smart-clickable:allow-pointer-events flex gap-2">
									<button
										v-if="
											(notif.body?.type === 'team_invite' ||
												notif.body?.type === 'organization_invite') &&
											!notif.read
										"
										class="iconified-button square-button brand-button [&>svg]:!mr-0"
										@click.stop.prevent="handleAcceptInvite(notif)"
									>
										<CheckIcon />
									</button>
									<button
										v-if="
											(notif.body?.type === 'team_invite' ||
												notif.body?.type === 'organization_invite') &&
											!notif.read
										"
										class="iconified-button square-button danger-button [&>svg]:!mr-0"
										@click.stop.prevent="handleDeclineInvite(notif)"
									>
										<XIcon />
									</button>
									<button
										v-else-if="!notif.read"
										class="iconified-button square-button [&>svg]:!mr-0"
										@click.stop.prevent="handleMarkAsRead(notif)"
									>
										<CheckIcon />
									</button>
								</div>
							</div>
						</SmartClickable>
					</div>
				</div>
			</template>
		</OverflowMenu>
	</ButtonStyled>
</template>

<script setup>
import
	{
		BellIcon,
		CalendarIcon,
		CheckCheckIcon,
		CheckIcon,
		DropdownIcon,
		HistoryIcon,
		ScaleIcon,
		UserPlusIcon,
		VersionIcon,
		XIcon,
	} from '@modrinth/assets'
import
	{
		Avatar,
		ButtonStyled,
		defineMessages,
		DoubleIcon,
		OverflowMenu,
		SmartClickable,
		useRelativeTime,
		useVIntl,
	} from '@modrinth/ui'

import
	{
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

// Auto-refresh
const REFRESH_INTERVAL = 60000 // 1 minute

const notificationsOverflow = ref(null)

const refreshInterval = setInterval(() => {
	if (notificationsOverflow.value) {
		refreshNotifications()
	}
}, REFRESH_INTERVAL)

onBeforeUnmount(() => {
	clearInterval(refreshInterval)
})

const messages = defineMessages({
	viewAllNotifications: {
		id: 'layout.notifications.view-all',
		defaultMessage: 'View all',
	},
	markAllAsRead: {
		id: 'layout.notifications.mark-all-read',
		defaultMessage: 'Mark all as read',
	},
	viewHistory: {
		id: 'layout.notifications.view-history',
		defaultMessage: 'View history',
	},
	noUnreadNotifications: {
		id: 'layout.notifications.no-unread',
		defaultMessage: 'No unread notifications',
	},
})

async function handleAcceptInvite(notif) {
	try {
		// Mark as read locally immediately
		if (notificationsData.value) {
			const n = notificationsData.value.find((n) => n.id === notif.id)
			if (n) n.read = true
		}

		await acceptTeamInvite(notif.body.team_id)
		markAsRead([notif.id]).catch((err) => {
			console.error('Error marking as read:', err)
		})
	} catch (err) {
		console.error('Error accepting invite:', err)
	}
}

async function handleDeclineInvite(notif) {
	try {
		// Mark as read locally immediately
		if (notificationsData.value) {
			const n = notificationsData.value.find((n) => n.id === notif.id)
			if (n) n.read = true
		}

		await removeSelfFromTeam(notif.body.team_id)
		markAsRead([notif.id]).catch((err) => {
			console.error('Error marking as read:', err)
		})
	} catch (err) {
		console.error('Error declining invite:', err)
	}
}

async function handleMarkAsRead(notif) {
	try {
		const ids = [notif.id, ...(notif.grouped_notifs ? notif.grouped_notifs.map((n) => n.id) : [])]

		// Mark as read locally immediately
		if (notificationsData.value) {
			for (const id of ids) {
				const n = notificationsData.value.find((n) => n.id === id)
				if (n) n.read = true
			}
		}

		markAsRead(ids).catch((err) => {
			console.error('Error marking as read:', err)
		})
	} catch (err) {
		console.error('Error marking as read:', err)
	}
}

function handleViewAllNotifications() {
	notificationsOverflow.value?.close()
	router.push('/dashboard/notifications')
}

function handleViewHistory() {
	notificationsOverflow.value?.close()
	router.push('/dashboard/notifications/history')
}

async function handleNotificationClick(notif) {
	notificationsOverflow.value?.close()
	if (!notif.read) {
		handleMarkAsRead(notif)
	}
}

async function handleMarkAllAsRead() {
	try {
		const ids = notificationsData.value?.map((n) => n.id) || []

		// Mark all as read locally immediately
		if (notificationsData.value) {
			for (const n of notificationsData.value) {
				n.read = true
			}
		}

		markAsRead(ids).catch((err) => {
			console.error('Error marking all as read:', err)
		})

		notificationsOverflow.value?.close()
	} catch (err) {
		console.error('Error marking all as read:', err)
	}
}
</script>
