<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Dashboard<span class="beta-badge">BETA</span></h1>
        <NavStack>
          <NavStackItem link="/dashboard" label="Overview">
            <DashboardIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/projects" label="Projects">
            <ListIcon />
          </NavStackItem>
          <!--          <NavStackItem link="/dashboard/analytics" label="Analytics">-->
          <!--            <ChartIcon />-->
          <!--          </NavStackItem>-->
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
import NavStack from '~/components/ui/NavStack'
import NavStackItem from '~/components/ui/NavStackItem'

import DashboardIcon from '~/assets/images/utils/dashboard.svg'
import CurrencyIcon from '~/assets/images/utils/currency.svg'
import ListIcon from '~/assets/images/utils/list.svg'

definePageMeta({
  middleware: 'auth',
})

const route = useRoute()
</script>
<style lang="scss" scoped></style>
