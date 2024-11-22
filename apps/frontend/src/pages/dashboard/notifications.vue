<template>
  <div>
    <section class="universal-card">
      <Breadcrumbs
        v-if="history"
        current-title="历史通知"
        :link-stack="[{ href: `/dashboard/notifications`, label: '通知' }]"
      />
      <div class="header__row">
        <div class="header__title">
          <h2 v-if="history" class="text-2xl">历史通知</h2>
          <h2 v-else class="text-2xl">通知</h2>
        </div>
        <template v-if="!history">
          <Button v-if="hasRead" @click="updateRoute()"> <HistoryIcon /> 查看历史通知 </Button>
          <Button v-if="notifications.length > 0" color="danger" @click="readAll()">
            <CheckCheckIcon /> 全部标记为已读
          </Button>
        </template>
      </div>
      <Chips
        v-if="notifTypes.length > 1"
        v-model="selectedType"
        :items="notifTypes"
        :format-label="(x) => {
          switch(x){
            case 'all': {
              return '全部'
            }
            case 'status_change': {
              return '状态变更'
            }
            case 'moderator_message': {
              return '版主消息'
            }
            default: {
              return x
            }
          }
        }"
        :capitalize="false"
      />
      <p v-if="pending">加载中...</p>
      <template v-else-if="error">
        <p>异常加载通知:</p>
        <pre>
          {{ error }}
        </pre>
      </template>
      <template v-else-if="notifications && notifications.length > 0">
        <NotificationItem
          v-for="notification in notifications"
          :key="notification.id"
          :notifications="notifications"
          class="universal-card recessed"
          :notification="notification"
          :auth="auth"
          raised
          @update:notifications="() => refresh()"
        />
      </template>
      <p v-else>你没有收到任何消息.</p>
      <Pagination :page="page" :count="pages" @switch-page="changePage" />
    </section>
  </div>
</template>
<script setup>
import { Button } from "@modrinth/ui";
import { HistoryIcon } from "@modrinth/assets";
import {
  fetchExtraNotificationData,
  groupNotifications,
  markAsRead,
} from "~/helpers/notifications.js";
import NotificationItem from "~/components/ui/NotificationItem.vue";
import Chips from "~/components/ui/Chips.vue";
import CheckCheckIcon from "~/assets/images/utils/check-check.svg?component";
import Breadcrumbs from "~/components/ui/Breadcrumbs.vue";
import Pagination from "~/components/ui/Pagination.vue";

useHead({
  title: "通知 - BBSMC",
});

const auth = await useAuth();

const route = useNativeRoute();
const router = useNativeRouter();

const history = computed(() => {
  return route.name === "dashboard-notifications-history";
});

const selectedType = ref("all");
const page = ref(1);

const perPage = ref(50);

const { data, pending, error, refresh } = await useAsyncData(
  async () => {
    const pageNum = page.value - 1;

    const notifications = await useBaseFetch(`user/${auth.value.user.id}/notifications`);
    const showRead = history.value;
    const hasRead = notifications.some((notif) => notif.read);

    const types = [
      ...new Set(
        notifications
          .filter((notification) => {
            return showRead || !notification.read;
          })
          .map((notification) => notification.type),
      ),
    ];

    const filteredNotifications = notifications.filter(
      (notification) =>
        (selectedType.value === "all" || notification.type === selectedType.value) &&
        (showRead || !notification.read),
    );
    const pages = Math.ceil(filteredNotifications.length / perPage.value);

    return fetchExtraNotificationData(
      filteredNotifications.slice(pageNum * perPage.value, perPage.value + pageNum * perPage.value),
    ).then((notifications) => {
      return {
        notifications,
        types: types.length > 1 ? ["all", ...types] : types,
        pages,
        hasRead,
      };
    });
  },
  { watch: [page, history, selectedType] },
);

const notifications = computed(() => {
  if (data.value === null) {
    return [];
  }
  return groupNotifications(data.value.notifications, history.value);
});
const notifTypes = computed(() => data.value.types);
const pages = computed(() => data.value.pages);
const hasRead = computed(() => data.value.hasRead);

function updateRoute() {
  if (history.value) {
    router.push("/dashboard/notifications");
  } else {
    router.push("/dashboard/notifications/history");
  }
  selectedType.value = "all";
  page.value = 1;
}

async function readAll() {
  const ids = notifications.value.flatMap((notification) => [
    notification.id,
    ...(notification.grouped_notifs ? notification.grouped_notifs.map((notif) => notif.id) : []),
  ]);

  const updateNotifs = await markAsRead(ids);
  allNotifs.value = updateNotifs(allNotifs.value);
}

function changePage(newPage) {
  page.value = newPage;
  if (import.meta.client) {
    window.scrollTo({ top: 0, behavior: "smooth" });
  }
}
</script>
<style lang="scss" scoped>
.read-toggle-input {
  display: flex;
  align-items: center;
  gap: var(--spacing-card-md);

  .label__title {
    margin: 0;
  }
}

.header__title {
  h2 {
    margin: 0 auto 0 0;
  }
}
</style>
