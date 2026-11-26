<template>
	<section class="flex flex-col gap-4 pb-20">
		<BackToParentLink v-if="history" to="/dashboard/notifications" class="mt-2.5">
			{{ formatMessage(commonMessages.notificationsLabel) }}
		</BackToParentLink>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h2 class="m-0 text-xl font-semibold text-contrast md:text-2xl">
				{{
					formatMessage(
						history ? messages.notificationHistoryTitle : commonMessages.notificationsLabel,
					)
				}}
			</h2>
			<div v-if="!history && (data.hasRead || notifications.length)" class="flex flex-wrap gap-2">
				<Button v-if="data.hasRead" @click="updateRoute()">
					<HistoryIcon />
					{{ formatMessage(messages.viewHistory) }}
				</Button>
				<Button v-if="notifications.length > 0" type="colored" color="red" @click="readAll()">
					<CheckCheckIcon />
					{{ formatMessage(messages.markAllAsRead) }}
				</Button>
			</div>
		</div>
		<Chips
			v-if="notifTypes.length > 1"
			v-model="selectedType"
			:items="notifTypes"
			:format-label="(x) => (x === 'all' ? 'All' : formatProjectType(x).replace('_', ' ') + 's')"
			:capitalize="false"
		/>
		<p v-if="isPending" class="m-0">{{ formatMessage(messages.loadingNotifications) }}</p>
		<template v-else-if="error">
			<p class="m-0">{{ formatMessage(messages.errorLoadingNotifications) }}</p>
			<pre class="m-0 overflow-x-auto">{{ error }}</pre>
		</template>
		<template v-else-if="notifications.length > 0">
			<NotificationItem
				v-for="notification in notifications"
				:key="notification.id"
				:notifications="notifications"
				class="universal-card !mb-0"
				:notification="notification"
				:auth="auth"
				raised
				@update:notifications="() => refetch()"
			/>
		</template>
		<p v-else class="m-0">{{ formatMessage(messages.noUnreadNotifications) }}</p>
		<div v-if="pages > 1" class="flex justify-end">
			<Pagination :page="page" :count="pages" @switch-page="changePage" />
		</div>
	</section>
</template>
<script setup>
import { CheckCheckIcon, HistoryIcon } from '@modrinth/assets'
import {
	BackToParentLink,
	Button,
	Chips,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	Pagination,
	useVIntl,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'

import NotificationItem from '~/components/ui/NotificationItem.vue'
import {
	fetchExtraNotificationData,
	groupNotifications,
	markAsRead,
} from '~/helpers/platform-notifications.ts'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	notificationHistoryTitle: {
		id: 'dashboard.overview.notifications.history.title',
		defaultMessage: 'Notification history',
	},
	viewHistory: {
		id: 'dashboard.overview.notifications.button.view-history',
		defaultMessage: 'View history',
	},
	markAllAsRead: {
		id: 'dashboard.overview.notifications.button.mark-all-as-read',
		defaultMessage: 'Mark all as read',
	},
	loadingNotifications: {
		id: 'dashboard.overview.notifications.loading',
		defaultMessage: 'Loading notifications...',
	},
	errorLoadingNotifications: {
		id: 'dashboard.overview.notifications.error.loading',
		defaultMessage: 'Error loading notifications:',
	},
	noUnreadNotifications: {
		id: 'dashboard.overview.notifications.empty.no-unread',
		defaultMessage: "You don't have any unread notifications.",
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
		const visibleNotifications = notifications.filter(
			(n) => !n.type?.startsWith('shared_instance_'),
		)

		const typesInFeed = [
			...new Set(visibleNotifications.filter((n) => showRead || !n.read).map((n) => n.type)),
		]

		const filtered = visibleNotifications.filter(
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
			hasRead: visibleNotifications.some((n) => n.read),
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
