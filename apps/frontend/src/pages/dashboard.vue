<template>
	<div class="normal-page !mt-8">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ type: 'heading', label: 'Dashboard' },
					{ link: '/dashboard', label: 'Overview', icon: DashboardIcon },
					{ link: '/dashboard/notifications', label: 'Notifications', icon: NotificationsIcon },
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
import { commonMessages } from '@modrinth/ui'
import { type User, UserBadge } from '@modrinth/utils'

import NavStack from '~/components/ui/NavStack.vue'

const auth = (await useAuth()) as Ref<{ user: User | null }>

const isAffiliate = computed(() => {
	return auth.value.user && auth.value.user.badges & UserBadge.AFFILIATE
})

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

const route = useNativeRoute()
</script>
