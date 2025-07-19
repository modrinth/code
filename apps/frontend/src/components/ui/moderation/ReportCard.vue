<template>
  <div class="universal-card">
    <div class="flex w-full flex-row items-center justify-between">
      <span class="text-md flex items-center gap-2">
        Reported for
        <span class="rounded-full align-middle font-semibold text-contrast">
          {{ formattedReportType }}
        </span>
        By
        <nuxt-link
          :to="`/user/${report.reporter_user.username}`"
          class="inline-flex flex-row items-center gap-1 transition-colors duration-100 ease-in-out hover:text-brand"
        >
          <Avatar :src="report.reporter_user.avatar_url" circle size="1.75rem" />
          <span>{{ report.reporter_user.username }}</span>
        </nuxt-link>
      </span>
      <div class="flex flex-row items-center gap-2">
        <span class="text-md text-secondary">{{ formatRelativeTime(report.created) }}</span>
        <ButtonStyled v-if="visibleQuickReplies.length > 0" circular>
          <OverflowMenu :options="visibleQuickReplies"
            >Quick Reply <ChevronDownIcon />
          </OverflowMenu>
        </ButtonStyled>
        <ButtonStyled circular>
          <OverflowMenu :options="quickActions"
            ><template #default> <EllipsisVerticalIcon /></template
            ><template #copy-id> <ClipboardCopyIcon />Copy ID </template
            ><template #copy-link> <LinkIcon /> Copy link </template></OverflowMenu
          >
        </ButtonStyled>
      </div>
    </div>
    <hr class="my-4 rounded-xl border-solid text-divider" />
    <div class="flex flex-col gap-4">
      <div class="flex items-center gap-3">
        <Avatar :src="reportItemAvatarUrl" :circle="report.item_type === 'user'" size="3rem" />
        <div class="min-w-0 flex-1">
          <span class="truncate text-lg font-semibold">{{ reportItemTitle }}</span>
          <div class="flex items-center gap-2 text-sm text-secondary">
            <nuxt-link
              v-if="report.target && report.item_type != 'user'"
              :to="`/${report.target.type}/${report.target.slug}`"
              class="inline-flex flex-row items-center gap-1 transition-colors duration-100 ease-in-out hover:text-brand"
            >
              <Avatar
                :src="report.target?.avatar_url"
                :circle="report.target.type === 'user'"
                size="1rem"
              />
              <span
                ><OrganizationIcon
                  v-if="report.target.type === 'organization'"
                  class="align-middle"
                />
                {{ report.target.name || "Unknown User" }}</span
              >
            </nuxt-link>
            <span
              class="my-auto rounded-full bg-button-bg p-0.5 px-2 align-middle text-xs font-semibold text-secondary"
              >{{ formattedItemType }}</span
            >
            <span v-if="report.item_type === 'version' && report.version" class="font-mono">
              {{ report.version.files.find((file) => file.primary)?.filename || "Unknown Version" }}
            </span>
          </div>
        </div>
        <div>
          <ButtonStyled circular>
            <nuxt-link :to="reportItemUrl">
              <EyeIcon />
            </nuxt-link>
          </ButtonStyled>
        </div>
      </div>
    </div>
    <CollapsibleRegion class="my-4">
      <div class="markdown-body" v-html="renderHighlightedString(report.body)"></div>
      <ReportThread
        v-if="report.thread"
        :thread="report.thread"
        :report="report"
        @update-thread="updateThread"
      />
    </CollapsibleRegion>
  </div>
</template>

<script setup lang="ts">
import {
  Avatar,
  useRelativeTime,
  OverflowMenu,
  type OverflowMenuOption,
  CollapsibleRegion,
  ButtonStyled,
} from "@modrinth/ui";
import {
  EllipsisVerticalIcon,
  OrganizationIcon,
  EyeIcon,
  ClipboardCopyIcon,
  LinkIcon,
} from "@modrinth/assets";
import { renderHighlightedString } from "@modrinth/utils";
import {
  type ExtendedReport,
  reportQuickReplies,
  type ReportQuickReply,
} from "@modrinth/moderation";
import ChevronDownIcon from "../servers/icons/ChevronDownIcon.vue";
import ReportThread from "../thread/ReportThread.vue";

const props = defineProps<{
  report: ExtendedReport;
}>();

const formatRelativeTime = useRelativeTime();

function updateThread(newThread: any) {
  if (props.report.thread) {
    Object.assign(props.report.thread, newThread);
  }
}

const quickActions: OverflowMenuOption[] = [
  {
    id: "copy-link",
    action: () => {
      const base = window.location.origin;
      const reportUrl = `${base}/moderation/reports?q=${props.report.id}`;
      navigator.clipboard.writeText(reportUrl).then(() => {
        addNotification({
          type: "success",
          title: "Report link copied",
          text: "The link to this report has been copied to your clipboard.",
        });
      });
    },
  },
  {
    id: "copy-id",
    action: () => {
      navigator.clipboard.writeText(props.report.id).then(() => {
        addNotification({
          type: "success",
          title: "Report ID copied",
          text: "The ID of this report has been copied to your clipboard.",
        });
      });
    },
  },
];

const visibleQuickReplies = computed<OverflowMenuOption[]>(() => {
  return reportQuickReplies
    .filter((reply) => {
      if (reply.shouldShow === undefined) return true;
      if (typeof reply.shouldShow === "function") {
        return reply.shouldShow(props.report);
      }

      return reply.shouldShow;
    })
    .map(
      (reply) =>
        ({
          id: reply.label,
          action: () => handleQuickReply(reply),
        }) as OverflowMenuOption,
    );
});

function handleQuickReply(reply: ReportQuickReply) {
  console.log(`Quick reply sent: ${reply.message}`);
}

const reportItemAvatarUrl = computed(() => {
  switch (props.report.item_type) {
    case "project":
    case "version":
      return props.report.project?.icon_url || "";
    case "user":
      return props.report.user?.avatar_url || "";
    default:
      return undefined;
  }
});

const reportItemTitle = computed(() => {
  if (props.report.item_type === "user") return props.report.user?.username || "Unknown User";

  return props.report.project?.title || "Unknown Project";
});

const reportItemUrl = computed(() => {
  switch (props.report.item_type) {
    case "user":
      return `/user/${props.report.user?.username}`;
    case "project":
      return `/${props.report.project?.project_type}/${props.report.project?.slug}`;
    case "version":
      return `/${props.report.project?.project_type}/${props.report.project?.slug}/versions/${props.report.version?.id}`;
  }
});

const formattedItemType = computed(() => {
  const itemType = props.report.item_type;
  return itemType.charAt(0).toUpperCase() + itemType.slice(1);
});

const formattedReportType = computed(() => {
  const reportType = props.report.report_type;

  // some are split by -, some are split by " "
  const words = reportType.includes("-") ? reportType.split("-") : reportType.split(" ");
  return words.map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(" ");
});
</script>

<style lang="scss" scoped></style>
