<template>
	<div class="normal-page !mt-8">
		<div class="normal-page__sidebar">
			<NavStack
				:items="
					[
						{ type: 'heading', label: 'Dashboard' },
						{ link: '/dashboard', label: 'Overview', icon: DashboardIcon },
						{
							link: '/dashboard/notifications',
							label: 'Notifications',
							icon: NotificationsIcon,
							badge: unreadNotificationsCount > 0 ? `${unreadNotificationsCount}` : undefined,
						},
						{ link: '/dashboard/reports', label: 'Active reports', icon: ReportIcon },
						{
							link: '/dashboard/collections',
							label: formatMessage(commonMessages.collectionsLabel),
							icon: LibraryIcon,
						},
						{ type: 'heading', label: 'Creators' },
						{ link: '/dashboard/projects', label: 'Projects', icon: ListIcon },
						{ link: '/dashboard/organizations', label: 'Organizations', icon: OrganizationIcon },
						{ link: '/dashboard/analytics', label: 'Analytics', icon: ChartIcon },
						{
							link: '/dashboard/affiliate-links',
							label: formatMessage(commonMessages.affiliateLinksButton),
							icon: AffiliateIcon,
							shown: isAffiliate,
						},
						{ link: '/dashboard/revenue', label: 'Revenue', icon: CurrencyIcon, matchNested: true },
					].filter(Boolean)
				"
			/>
		</div>
		<div class="normal-page__content mt-4 lg:!mt-0">
			<NuxtPage :route="route" />
		</div>
	</div>
</template>
<script setup lang="ts">
import {
	AffiliateIcon,
	BellIcon as NotificationsIcon,
	ChartIcon,
	CurrencyIcon,
	DashboardIcon,
	LibraryIcon,
	ListIcon,
	OrganizationIcon,
	ReportIcon,
} from '@modrinth/assets'
import { commonMessages, useVIntl } from '@modrinth/ui'
import { type User, UserBadge } from '@modrinth/utils'

import NavStack from '~/components/ui/NavStack.vue'
import {
	fetchExtraNotificationData,
	groupNotifications,
} from '~/helpers/platform-notifications.ts'

const auth = (await useAuth()) as Ref<{ user: User | null }>

const isAffiliate = computed(() => {
	return auth.value.user && auth.value.user.badges & UserBadge.AFFILIATE
})

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

useSeoMeta({
	robots: 'noindex',
})

const route = useNativeRoute()

const { data: notificationsData } = await useAsyncData(
	'dashboard-nav-notifications',
	async () => {
		if (!auth.value.user) return null
		const notifs = await useBaseFetch(`user/${auth.value.user.id}/notifications`)
		return await fetchExtraNotificationData(notifs)
	},
	{
		watch: [auth],
	},
)

const unreadNotificationsCount = computed(() => {
	if (!notificationsData.value) return 0
	const grouped = groupNotifications(
		notificationsData.value.filter((n) => !n.read),
		false,
	)
	return grouped.length
})
</script>
