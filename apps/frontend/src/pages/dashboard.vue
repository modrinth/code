<template>
	<div class="normal-page">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ link: '/dashboard', label: 'Overview', icon: DashboardIcon },
					{ link: '/dashboard/notifications', label: 'Notifications', icon: NotificationsIcon },
					{ link: '/dashboard/reports', label: 'Active reports', icon: ReportIcon },
					{ link: '/dashboard/analytics', label: 'Analytics', icon: ChartIcon },
					{ type: 'separator' },
					{ link: '/dashboard/projects', label: 'Projects', icon: ListIcon },
					{ link: '/dashboard/organizations', label: 'Organizations', icon: OrganizationIcon },
					{
						link: '/dashboard/collections',
						label: formatMessage(commonMessages.collectionsLabel),
						icon: LibraryIcon,
					},
					{ link: '/dashboard/revenue', label: 'Revenue', icon: CurrencyIcon },
				]"
			/>
		</div>
		<div class="normal-page__content">
			<NuxtPage :route="route" />
		</div>
	</div>
</template>
<script setup>
import {
	ChartIcon,
	CurrencyIcon,
	DashboardIcon,
	LibraryIcon,
	ListIcon,
	BellIcon as NotificationsIcon,
	OrganizationIcon,
	ReportIcon,
} from '@modrinth/assets'
import { commonMessages } from '@modrinth/ui'

import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

const route = useNativeRoute()
</script>
