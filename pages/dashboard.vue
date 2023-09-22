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
          <NavStackItem link="/dashboard/follows" label="Followed projects">
            <HeartIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/reports" label="Active reports">
            <ReportIcon />
          </NavStackItem>

          <h3>Manage</h3>
          <NavStackItem v-if="true" link="/dashboard/projects" label="Projects">
            <ListIcon />
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
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

import DashboardIcon from '~/assets/images/utils/dashboard.svg'
import CurrencyIcon from '~/assets/images/utils/currency.svg'
import ListIcon from '~/assets/images/utils/list.svg'
import ReportIcon from '~/assets/images/utils/report.svg'
import NotificationsIcon from '~/assets/images/utils/bell.svg'
import HeartIcon from '~/assets/images/utils/heart.svg'

definePageMeta({
  middleware: 'auth',
})

const route = useRoute()
</script>
