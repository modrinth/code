<template>
	<div>
		<section class="universal-card dashboard-header">
			<Avatar :src="auth.user.avatar_url" size="md" circle :alt="auth.user.username" />
			<div class="username">
				<h1>
					{{ auth.user.username }}
				</h1>
				<NuxtLink class="goto-link" :to="`/user/${auth.user.username}`">
					{{ formatMessage(commonMessages.visitYourProfile) }}
					<ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
				</NuxtLink>
			</div>
		</section>
		<div>
			<section class="universal-card">
				<div class="header__row">
					<h2 class="header__title text-2xl">
						{{ formatMessage(commonMessages.notificationsLabel) }}
					</h2>
					<nuxt-link
						v-if="notifications.length > 0"
						class="goto-link"
						to="/dashboard/notifications"
					>
						{{ formatMessage(messages.seeAll) }}
						<ChevronRightIcon />
					</nuxt-link>
				</div>
				<template v-if="notifications.length > 0">
					<NotificationItem
						v-for="notification in notifications"
						:key="notification.id"
						:notifications="notifications"
						class="universal-card recessed"
						:notification="notification"
						:auth="auth"
						raised
						compact
						@update:notifications="() => refetch()"
					/>
					<nuxt-link
						v-if="extraNotifs > 0"
						class="goto-link view-more-notifs mt-4"
						to="/dashboard/notifications"
					>
						{{ formatMessage(messages.viewMore, { extraNotifs: extraNotifs }) }}
						<ChevronRightIcon />
					</nuxt-link>
				</template>
				<div v-else class="universal-body">
					<p>{{ formatMessage(messages.noUnreadNotifications) }}</p>
					<ButtonStyled>
						<nuxt-link to="/dashboard/notifications/history" class="!mt-4 w-fit">
							<HistoryIcon />
							{{ formatMessage(messages.viewNotificationHistory) }}
						</nuxt-link>
					</ButtonStyled>
				</div>
			</section>
		</div>
	</div>
</template>
<script setup>
import { ChevronRightIcon, HistoryIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import NotificationItem from '~/components/ui/NotificationItem.vue'
import { fetchExtraNotificationData, groupNotifications } from '~/helpers/platform-notifications.ts'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	headTitle: {
		id: 'dashboard.head-title',
		defaultMessage: 'Dashboard',
	},
	seeAll: {
		id: 'dashboard.notifications.link.see-all',
		defaultMessage: 'See all',
	},
	viewMore: {
		id: 'dashboard.notifications.link.view-more',
		defaultMessage:
			'View {extraNotifs} more {extraNotifs, plural, one {notification} other {notifications}}',
	},
	noUnreadNotifications: {
		id: 'dashboard.notifications.empty.no-unread',
		defaultMessage: 'You have no unread notifications.',
	},
	viewNotificationHistory: {
		id: 'dashboard.notifications.link.view-history',
		defaultMessage: 'View notification history',
	},
	totalDownloads: {
		id: 'dashboard.analytics.total-downloads',
		defaultMessage: 'Total downloads',
	},
	totalFollowers: {
		id: 'dashboard.analytics.total-followers',
		defaultMessage: 'Total followers',
	},
	fromProjects: {
		id: 'dashboard.analytics.from-projects',
		defaultMessage: 'from {count} {count, plural, one {project} other {projects}}',
	},
})

useHead({
	title: () => `${formatMessage(messages.headTitle)} - Modrinth`,
})

const auth = await useAuth()
const client = injectModrinthClient()

const { data, refetch } = useQuery({
	queryKey: computed(() => ['user', auth.value?.user?.id, 'notifications']),
	queryFn: async () => {
		const notifications = await client.labrinth.notifications_v2.getUserNotifications(
			auth.value?.user?.id,
		)

		const filteredNotifications = notifications.filter((notif) => !notif.read)
		const slice = filteredNotifications.slice(0, 30)

		return fetchExtraNotificationData(client, slice).then((notifications) => {
			notifications = groupNotifications(notifications).slice(0, 3)
			return { notifications, extraNotifs: filteredNotifications.length - slice.length }
		})
	},
	enabled: computed(() => !!auth.value?.user?.id),
})

const notifications = computed(() => {
	return data.value?.notifications ?? []
})

const extraNotifs = computed(() => (data.value ? data.value.extraNotifs : 0))
</script>
<style lang="scss">
.dashboard-header {
	display: flex;
	gap: var(--spacing-card-bg);
	grid-area: header;

	.username {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-card-sm);
		justify-content: center;
		word-break: break-word;

		h1 {
			margin: 0;
		}
	}

	@media screen and (max-width: 650px) {
		.avatar {
			width: 4rem;
			height: 4rem;
		}

		.username {
			h1 {
				font-size: var(--font-size-xl);
			}
		}
	}
}
</style>
