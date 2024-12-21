<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Dashboard</h1>
        <NavStack>
          <NavStackItem link="/dashboard" label="Overview">
            <DashboardIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/notifications" label="Notifications">
            <NotificationsIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/reports" label="Active reports">
            <ReportIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/analytics" label="Analytics">
            <ChartIcon aria-hidden="true" />
          </NavStackItem>

          <h3>Manage</h3>
          <NavStackItem v-if="true" link="/dashboard/projects" label="Projects">
            <ListIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem v-if="true" link="/dashboard/organizations" label="Organizations">
            <OrganizationIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            link="/dashboard/collections"
            :label="formatMessage(commonMessages.collectionsLabel)"
          >
            <LibraryIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/revenue" label="Revenue">
            <CurrencyIcon aria-hidden="true" />
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
import {
  DashboardIcon,
  CurrencyIcon,
  ListIcon,
  ReportIcon,
  BellIcon as NotificationsIcon,
  OrganizationIcon,
  LibraryIcon,
  ChartIcon,
} from "@modrinth/assets";
import { commonMessages } from "@modrinth/ui";
import NavStack from "~/components/ui/NavStack.vue";
import NavStackItem from "~/components/ui/NavStackItem.vue";

const { formatMessage } = useVIntl();

definePageMeta({
  middleware: "auth",
});

const route = useNativeRoute();
</script>
