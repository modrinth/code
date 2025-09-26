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

import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

const route = useNativeRoute()
</script>
