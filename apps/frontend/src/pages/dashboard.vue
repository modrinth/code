<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>仪表盘</h1>
        <NavStack>
          <NavStackItem link="/dashboard" label="主页">
            <DashboardIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/notifications" label="通知">
            <NotificationsIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/reports" label="活跃报告">
            <ReportIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem link="/dashboard/analytics" label="统计">
            <ChartIcon aria-hidden="true" />
          </NavStackItem>

          <h3>管理</h3>
          <NavStackItem v-if="true" link="/dashboard/projects" label="我的资源">
            <ListIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem v-if="true" link="/dashboard/organizations" label="团队">
            <OrganizationIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            link="/dashboard/collections"
            :label="formatMessage(commonMessages.collectionsLabel)"
          >
            <LibraryIcon aria-hidden="true" />
          </NavStackItem>
<!--          <NavStackItem link="/dashboard/revenue" label="收入">-->
<!--            <CurrencyIcon aria-hidden="true" />-->
<!--          </NavStackItem>-->
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtPage :route="route" />
    </div>
  </div>
</template>
<script setup>
import { LibraryIcon, ChartIcon } from "@modrinth/assets";
import NavStack from "~/components/ui/NavStack.vue";
import NavStackItem from "~/components/ui/NavStackItem.vue";

import DashboardIcon from "~/assets/images/utils/dashboard.svg?component";
import CurrencyIcon from "~/assets/images/utils/currency.svg?component";
import ListIcon from "~/assets/images/utils/list.svg?component";
import ReportIcon from "~/assets/images/utils/report.svg?component";
import NotificationsIcon from "~/assets/images/utils/bell.svg?component";
import OrganizationIcon from "~/assets/images/utils/organization.svg?component";

const { formatMessage } = useVIntl();

definePageMeta({
  middleware: "auth",
});

const route = useNativeRoute();
</script>
