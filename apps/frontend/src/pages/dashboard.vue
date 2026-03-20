<template>
	<div class="normal-page !mt-8">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ type: 'heading', label: formatMessage(messages.dashboard) },
					{ link: '/dashboard', label: formatMessage(messages.overview), icon: DashboardIcon },
					{ link: '/dashboard/notifications', label: formatMessage(messages.notifications), icon: NotificationsIcon },
					{ link: '/dashboard/reports', label: formatMessage(messages.activeReports), icon: ReportIcon },
					{
						link: '/dashboard/collections',
						label: formatMessage(commonMessages.collectionsLabel),
						icon: LibraryIcon,
					},
					{ type: 'heading', label: formatMessage(messages.creators) },
					{ link: '/dashboard/projects', label: formatMessage(messages.projects), icon: ListIcon },
					{ link: '/dashboard/organizations', label: formatMessage(messages.organizations), icon: OrganizationIcon },
					{ link: '/dashboard/analytics', label: formatMessage(messages.analytics), icon: ChartIcon },
					{
						link: '/dashboard/affiliate-links',
						label: formatMessage(commonMessages.affiliateLinksButton),
						icon: AffiliateIcon,
						shown: !!isAffiliate,
					},
					{ link: '/dashboard/revenue', label: formatMessage(messages.revenue), icon: CurrencyIcon, matchNested: true },
				]"
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
import { defineMessages, commonMessages, useVIntl } from '@modrinth/ui'
import { type User, UserBadge } from '@modrinth/utils'

import NavStack from '~/components/ui/NavStack.vue'

const auth = (await useAuth()) as Ref<{ user: User | null }>

const isAffiliate = computed(() => {
	return auth.value.user && auth.value.user.badges & UserBadge.AFFILIATE
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	dashboard: {
		id: 'dashboard.sidebar.label.dashboard',
		defaultMessage: 'Dashboard',
	},
	overview: {
		id: 'dashboard.sidebar.label.overview',
		defaultMessage: 'Overview',
	},
	notifications: {
		id: 'dashboard.sidebar.label.notifications',
		defaultMessage: 'Notifications',
	},
	activeReports: {
		id: 'dashboard.sidebar.label.activeReports',
		defaultMessage: 'Active reports',
	},
	creators: {
		id: 'dashboard.sidebar.label.creators',
		defaultMessage: 'Creators',
	},
	projects: {
		id: 'dashboard.sidebar.label.projects',
		defaultMessage: 'Projects',
	},
	organizations: {
		id: 'dashboard.sidebar.label.organizations',
		defaultMessage: 'Organizations',
	},
	analytics: {
		id: 'dashboard.sidebar.label.analytics',
		defaultMessage: 'Analytics',
	},
	revenue: {
		id: 'dashboard.sidebar.label.revenue',
		defaultMessage: 'Revenue',
	},
})

definePageMeta({
	middleware: 'auth',
})

useSeoMeta({
	robots: 'noindex',
})

const route = useNativeRoute()
</script>
