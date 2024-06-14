<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Dashboard</h1>
        <NavStack>
          <NavStackItem link="/dashboard" label="Overview">
            <DashboardIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/notifications" label="Notifications">
            <NotificationsIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/reports" label="Active reports">
            <ReportIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/analytics" label="Analytics">
            <ChartIcon />
          </NavStackItem>

          <h3>Manage</h3>
          <NavStackItem v-if="true" link="/dashboard/projects" label="Projects">
            <ListIcon />
          </NavStackItem>
          <NavStackItem v-if="true" link="/dashboard/organizations" label="Organizations">
            <OrganizationIcon />
          </NavStackItem>
          <NavStackItem
            link="/dashboard/collections"
            :label="formatMessage(commonMessages.collectionsLabel)"
          >
            <LibraryIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/revenue" label="Revenue">
            <CurrencyIcon />
          </NavStackItem>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtPage :route="route" />
    </div>
  </div>
</template>
<script setup>
import { LibraryIcon, ChartIcon } from 'omorphia'
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

import DashboardIcon from '~/assets/images/utils/dashboard.svg?component'
import CurrencyIcon from '~/assets/images/utils/currency.svg?component'
import ListIcon from '~/assets/images/utils/list.svg?component'
import ReportIcon from '~/assets/images/utils/report.svg?component'
import NotificationsIcon from '~/assets/images/utils/bell.svg?component'
import OrganizationIcon from '~/assets/images/utils/organization.svg?component'

const { formatMessage } = useVIntl()

definePageMeta({
  middleware: 'auth',
})

const route = useNativeRoute()
</script>
