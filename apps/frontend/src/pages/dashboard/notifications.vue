<template>
	<div>
		<section class="universal-card">
			<Breadcrumbs
				v-if="history"
				:current-title="formatMessage(messages.historyLabel)"
				:link-stack="[
					{
						href: `/dashboard/notifications`,
						label: formatMessage(commonMessages.notificationsLabel),
					},
				]"
			/>
			<div class="header__row">
				<div class="header__title">
					<h2 v-if="history" class="text-2xl">
						{{ formatMessage(messages.notificationHistoryTitle) }}
					</h2>
					<h2 v-else class="text-2xl">
						{{ formatMessage(commonMessages.notificationsLabel) }}
					</h2>
				</div>
				<template v-if="!history">
					<Button v-if="data.hasRead" @click="updateRoute()">
						<HistoryIcon />
						{{ formatMessage(messages.viewHistory) }}
					</Button>
					<Button v-if="notifications.length > 0" color="danger" @click="readAll()">
						<CheckCheckIcon />
						{{ formatMessage(messages.markAllAsRead) }}
					</Button>
				</template>
			</div>
			<Chips
				v-if="notifTypes.length > 1"
				v-model="selectedType"
				:items="notifTypes"
				:format-label="formatNotificationTypeLabel"
				:capitalize="false"
			/>
			<p v-if="isPending">{{ formatMessage(messages.loadingNotifications) }}</p>
			<template v-else-if="error">
				<p>{{ formatMessage(messages.errorLoadingNotifications) }}</p>
				<pre>
          {{ error }}
        </pre>
			</template>
			<template v-else-if="notifications && notifications.length > 0">
				<NotificationItem
					v-for="notification in notifications"
					:key="notification.id"
					:notifications="notifications"
					class="universal-card recessed"
					:notification="notification"
					:auth="auth"
					raised
					@update:notifications="() => refetch()"
				/>
			</template>
			<p v-else>{{ formatMessage(messages.noUnreadNotifications) }}</p>
			<div class="flex justify-end">
				<Pagination :page="page" :count="pages" @switch-page="changePage" />
			</div>
		</section>
	</div>
</template>
<script setup>
import { CheckCheckIcon, HistoryIcon } from '@modrinth/assets'
import {
	Button,
	Chips,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	Pagination,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import NotificationItem from '~/components/ui/NotificationItem.vue'
import {
	fetchExtraNotificationData,
	groupNotifications,
	markAsRead,
} from '~/helpers/platform-notifications.ts'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	historyLabel: {
		id: 'dashboard.notifications.history.label',
		defaultMessage: 'History',
	},
	notificationHistoryTitle: {
		id: 'dashboard.notifications.history.title',
		defaultMessage: 'Notification history',
	},
	viewHistory: {
		id: 'dashboard.notifications.button.view-history',
		defaultMessage: 'View history',
	},
	markAllAsRead: {
		id: 'dashboard.notifications.button.mark-all-as-read',
		defaultMessage: 'Mark all as read',
	},
	loadingNotifications: {
		id: 'dashboard.notifications.loading',
		defaultMessage: 'Loading notifications...',
	},
	errorLoadingNotifications: {
		id: 'dashboard.notifications.error.loading',
		defaultMessage: 'Error loading notifications:',
	},
	noUnreadNotifications: {
		id: 'dashboard.notifications.empty.no-unread',
		defaultMessage: "You don't have any unread notifications.",
	},
	projectUpdatesType: {
		id: 'dashboard.notifications.type.project-updates',
		defaultMessage: 'Project updates',
	},
	teamInvitesType: {
		id: 'dashboard.notifications.type.team-invites',
		defaultMessage: 'Team invites',
	},
	organizationInvitesType: {
		id: 'dashboard.notifications.type.organization-invites',
		defaultMessage: 'Organization invites',
	},
	statusChangesType: {
		id: 'dashboard.notifications.type.status-changes',
		defaultMessage: 'Status changes',
	},
	moderatorMessagesType: {
		id: 'dashboard.notifications.type.moderator-messages',
		defaultMessage: 'Moderator messages',
	},
	otherNotificationsType: {
		id: 'dashboard.notifications.type.other',
		defaultMessage: 'Other notifications',
	},
})

const client = injectModrinthClient()
const auth = await useAuth()
const route = useNativeRoute()
const router = useNativeRouter()

const history = computed(() => route.name === 'dashboard-notifications-history')

useHead({
	title: () =>
		`${formatMessage(history.value ? messages.notificationHistoryTitle : commonMessages.notificationsLabel)} - Modrinth`,
})

const selectedType = ref('all')
const page = ref(1)
const perPage = ref(50)

function formatNotificationTypeLabel(type) {
	if (type === 'all') {
		return formatMessage(commonMessages.allProjectType)
	}

	const notificationTypeMessages = {
		project_update: messages.projectUpdatesType,
		team_invite: messages.teamInvitesType,
		organization_invite: messages.organizationInvitesType,
		status_change: messages.statusChangesType,
		moderator_message: messages.moderatorMessagesType,
	}

	return formatMessage(notificationTypeMessages[type] ?? messages.otherNotificationsType)
}

const { data, isPending, error, refetch } = useQuery({
	queryKey: computed(() => [
		'user',
		auth.value?.user?.id,
		'notifications',
		page.value,
		history.value,
		selectedType.value,
	]),
	queryFn: async () => {
		const pageNum = page.value - 1
		const showRead = history.value
		const notifications = await client.labrinth.notifications_v2.getUserNotifications(
			auth.value?.user?.id,
		)

		const typesInFeed = [
			...new Set(notifications.filter((n) => showRead || !n.read).map((n) => n.type)),
		]

		const filtered = notifications.filter(
			(n) =>
				(selectedType.value === 'all' || n.type === selectedType.value) && (showRead || !n.read),
		)

		const pages = Math.max(1, Math.ceil(filtered.length / perPage.value))

		return fetchExtraNotificationData(
			client,
			filtered.slice(pageNum * perPage.value, pageNum * perPage.value + perPage.value),
		).then((notifs) => ({
			notifications: notifs,
			notifTypes: typesInFeed.length > 1 ? ['all', ...typesInFeed] : typesInFeed,
			pages,
			hasRead: notifications.some((n) => n.read),
		}))
	},
	enabled: computed(() => !!auth.value?.user?.id),
	placeholderData: { notifications: [], notifTypes: [], pages: 1, hasRead: false },
})

const notifications = computed(() =>
	data.value ? groupNotifications(data.value.notifications) : [],
)

const notifTypes = computed(() => data.value?.notifTypes || [])
const pages = computed(() => data.value?.pages ?? 1)

function updateRoute() {
	router.push(history.value ? '/dashboard/notifications' : '/dashboard/notifications/history')
	selectedType.value = 'all'
	page.value = 1
}

async function readAll() {
	const ids = notifications.value.flatMap((n) => [
		n.id,
		...(n.grouped_notifs ? n.grouped_notifs.map((g) => g.id) : []),
	])

	await markAsRead(client, ids)
	await refetch()
}

function changePage(newPage) {
	page.value = newPage
	if (import.meta.client) window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>
<style lang="scss" scoped>
.read-toggle-input {
	display: flex;
	align-items: center;
	gap: var(--spacing-card-md);

	.label__title {
		margin: 0;
	}
}

.header__title {
	h2 {
		margin: 0 auto 0 0;
	}
}
</style>
