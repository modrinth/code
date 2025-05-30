<script setup lang="ts">
import dayjs from "dayjs";
import { ButtonStyled, commonMessages, CopyCode, ServerNotice, TagItem } from "@modrinth/ui";
import { EditIcon, SettingsIcon, TrashIcon } from "@modrinth/assets";
import { ServerNotice as ServerNoticeType } from "@modrinth/utils";
import { useRelativeTime } from "@modrinth/ui";
import {
  DISMISSABLE,
  getDismissableMetadata,
  NOTICE_LEVELS,
} from "@modrinth/ui/src/utils/notices.ts";
import { useVIntl } from "@vintl/vintl";

const { formatMessage } = useVIntl();
const formatRelativeTime = useRelativeTime();

const props = defineProps<{
  notice: ServerNoticeType;
}>();
</script>
<template>
  <div class="col-span-full grid grid-cols-subgrid gap-4 rounded-2xl bg-bg-raised p-4">
    <div class="col-span-full grid grid-cols-subgrid items-center gap-4">
      <div>
        <CopyCode :text="`${notice.id}`" />
      </div>
      <div class="text-sm">
        <span v-if="notice.announce_at">
          {{ dayjs(notice.announce_at).format("MMM D, YYYY [at] h:mm A") }} ({{
            formatRelativeTime(notice.announce_at)
          }})
        </span>
        <template v-else> Never begins </template>
      </div>
      <div class="text-sm">
        <span
          v-if="notice.expires"
          v-tooltip="dayjs(notice.expires).format('MMMM D, YYYY [at] h:mm A')"
        >
          {{ formatRelativeTime(notice.expires) }}
        </span>
        <template v-else> Never expires </template>
      </div>
      <div
        :style="
          NOTICE_LEVELS[notice.level]
            ? {
                '--_color': NOTICE_LEVELS[notice.level].colors.text,
                '--_bg-color': NOTICE_LEVELS[notice.level].colors.bg,
              }
            : undefined
        "
      >
        <TagItem>
          {{
            NOTICE_LEVELS[notice.level]
              ? formatMessage(NOTICE_LEVELS[notice.level].name)
              : notice.level
          }}
        </TagItem>
      </div>
      <div
        :style="{
          '--_color': getDismissableMetadata(notice.dismissable).colors.text,
          '--_bg-color': getDismissableMetadata(notice.dismissable).colors.bg,
        }"
      >
        <TagItem>
          {{ formatMessage(getDismissableMetadata(notice.dismissable).name) }}
        </TagItem>
      </div>
      <div class="col-span-2 flex gap-2 md:col-span-1">
        <ButtonStyled>
          <button @click="() => startEditing(notice)">
            <EditIcon /> {{ formatMessage(commonMessages.editButton) }}
          </button>
        </ButtonStyled>
        <ButtonStyled color="red">
          <button @click="() => deleteNotice(notice)">
            <TrashIcon /> {{ formatMessage(commonMessages.deleteLabel) }}
          </button>
        </ButtonStyled>
      </div>
    </div>
    <div class="col-span-full grid">
      <ServerNotice
        :level="notice.level"
        :message="notice.message"
        :dismissable="notice.dismissable"
        :title="notice.title"
        preview
      />
      <div class="mt-4 flex items-center gap-2">
        <span v-if="!notice.assigned || notice.assigned.length === 0"
          >Not assigned to any servers</span
        >
        <span v-else-if="!notice.assigned.some((n) => n.kind === 'server')">
          Assigned to
          {{ notice.assigned.filter((n) => n.kind === "node").length }} nodes
        </span>
        <span v-else-if="!notice.assigned.some((n) => n.kind === 'node')">
          Assigned to
          {{ notice.assigned.filter((n) => n.kind === "server").length }} servers
        </span>
        <span v-else>
          Assigned to
          {{ notice.assigned.filter((n) => n.kind === "server").length }} servers and
          {{ notice.assigned.filter((n) => n.kind === "node").length }} nodes
        </span>
        •
        <button
          class="m-0 flex items-center gap-1 border-none bg-transparent p-0 text-blue hover:underline hover:brightness-125 active:scale-95 active:brightness-150"
          @click="() => startEditing(notice, true)"
        >
          <SettingsIcon />
          Edit assignments
        </button>
      </div>
    </div>
  </div>
</template>
