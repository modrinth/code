<template>
  <div class="universal-card">
    <div class="flex w-full flex-row items-center justify-between">
      <span class="text-md flex items-center gap-2">
        Reported for
        <span
          class="my-auto rounded-full border-[2px] border-solid p-1 px-2 align-middle text-sm font-semibold text-secondary"
        >
          {{ formattedReportType }}
        </span>
        By
        <nuxt-link
          :to="`/user/${report.reporter_user.username}`"
          class="inline-flex flex-row items-center gap-1 transition-colors duration-100 ease-in-out hover:text-brand"
        >
          <Avatar :src="report.reporter_user.avatar_url" circle size="1.75rem" no-shadow />
          <span>{{ report.reporter_user.username }}</span>
        </nuxt-link>
      </span>
      <div class="flex flex-row items-center gap-2">
        <span class="text-md text-secondary">{{ formatRelativeTime(report.created) }}</span>
        <ButtonStyled circular>
          <OverflowMenu :options="quickReplyOptions">Quick Reply <ChevronDownIcon /> </OverflowMenu>
        </ButtonStyled>
        <ButtonStyled circular>
          <button>
            <EllipsisVerticalIcon />
          </button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Avatar,
  useRelativeTime,
  OverflowMenu,
  type OverflowMenuOption,
  ButtonStyled,
} from "@modrinth/ui";
import type { ExtendedReport } from "~/pages/moderation/reports.vue";
import ChevronDownIcon from "../servers/icons/ChevronDownIcon.vue";
import { EllipsisVerticalIcon } from "@modrinth/assets";

const props = defineProps<{
  report: ExtendedReport;
}>();

const formatRelativeTime = useRelativeTime();

interface QuickReply {
  label: string;
  message: string;
}

const quickReplies: readonly QuickReply[] = readonly([
  { label: "Invalid", message: "Your **report** is invalid. Please do not do this." },
  { label: "Duplicate", message: "This report is a duplicate." },
]);

const quickReplyOptions: OverflowMenuOption[] = quickReplies.map((reply) => ({
  id: reply.label,
  action: () => handleQuickReply(reply),
}));

function handleQuickReply(reply: QuickReply) {
  console.log(`Quick reply sent: ${reply.message}`);
}

const formattedReportType = computed(() => {
  const reportType = props.report.report_type;

  // some are split by -, some are split by " "
  const words = reportType.includes("-") ? reportType.split("-") : reportType.split(" ");
  return words.map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(" ");
});
</script>
