<script setup lang="ts">
import dayjs from "dayjs";
import {
  MoreVerticalIcon,
  HistoryIcon,
  DownloadIcon,
  SpinnerIcon,
  EditIcon,
  LockIcon,
  TrashIcon,
  FolderArchiveIcon,
  BotIcon,
  XIcon,
  LockOpenIcon,
  RotateCounterClockwiseIcon,
} from "@modrinth/assets";
import { ButtonStyled, commonMessages, OverflowMenu, ProgressBar } from "@modrinth/ui";
import { defineMessages, useVIntl } from "@vintl/vintl";
import { ref } from "vue";
import type { Backup } from "~/composables/pyroServers.ts";

const flags = useFeatureFlags();
const { formatMessage } = useVIntl();

const emit = defineEmits<{
  (e: "prepare" | "download" | "rename" | "restore" | "lock" | "retry"): void;
  (e: "delete", skipConfirmation?: boolean): void;
}>();

const props = withDefaults(
  defineProps<{
    backup: Backup;
    preview?: boolean;
    kyrosUrl?: string;
    jwt?: string;
  }>(),
  {
    preview: false,
    kyrosUrl: undefined,
    jwt: undefined,
  },
);

const backupQueued = computed(
  () =>
    props.backup.task?.create?.progress === 0 ||
    (props.backup.ongoing && !props.backup.task?.create),
);
const automated = computed(() => props.backup.automated);
const failedToCreate = computed(() => props.backup.interrupted);

const preparedDownloadStates = ["ready", "done"];
const inactiveStates = ["failed", "cancelled"];

const hasPreparedDownload = computed(() =>
  preparedDownloadStates.includes(props.backup.task?.file?.state ?? ""),
);

const creating = computed(() => {
  const task = props.backup.task?.create;
  if (task && task.progress < 1 && !inactiveStates.includes(task.state)) {
    return task;
  }
  if (props.backup.ongoing) {
    return {
      progress: 0,
      state: "ongoing",
    };
  }
  return undefined;
});

const restoring = computed(() => {
  const task = props.backup.task?.restore;
  if (task && task.progress < 1 && !inactiveStates.includes(task.state)) {
    return task;
  }
  return undefined;
});

const initiatedPrepare = ref(false);

const preparingFile = computed(() => {
  const task = props.backup.task?.file;
  return (
    (!task && initiatedPrepare.value) ||
    (task && task.progress < 1 && !inactiveStates.includes(task.state))
  );
});

const failedToRestore = computed(() => props.backup.task?.restore?.state === "failed");
const failedToPrepareFile = computed(() => props.backup.task?.file?.state === "failed");

const messages = defineMessages({
  locked: {
    id: "servers.backups.item.locked",
    defaultMessage: "Locked",
  },
  lock: {
    id: "servers.backups.item.lock",
    defaultMessage: "Lock",
  },
  unlock: {
    id: "servers.backups.item.unlock",
    defaultMessage: "Unlock",
  },
  restore: {
    id: "servers.backups.item.restore",
    defaultMessage: "Restore",
  },
  rename: {
    id: "servers.backups.item.rename",
    defaultMessage: "Rename",
  },
  queuedForBackup: {
    id: "servers.backups.item.queued-for-backup",
    defaultMessage: "Queued for backup",
  },
  preparingDownload: {
    id: "servers.backups.item.preparing-download",
    defaultMessage: "Preparing download...",
  },
  prepareDownload: {
    id: "servers.backups.item.prepare-download",
    defaultMessage: "Prepare download",
  },
  prepareDownloadAgain: {
    id: "servers.backups.item.prepare-download-again",
    defaultMessage: "Try preparing again",
  },
  alreadyPreparing: {
    id: "servers.backups.item.already-preparing",
    defaultMessage: "Already preparing backup for download",
  },
  creatingBackup: {
    id: "servers.backups.item.creating-backup",
    defaultMessage: "Creating backup...",
  },
  restoringBackup: {
    id: "servers.backups.item.restoring-backup",
    defaultMessage: "Restoring from backup...",
  },
  failedToCreateBackup: {
    id: "servers.backups.item.failed-to-create-backup",
    defaultMessage: "Failed to create backup",
  },
  failedToRestoreBackup: {
    id: "servers.backups.item.failed-to-restore-backup",
    defaultMessage: "Failed to restore from backup",
  },
  failedToPrepareFile: {
    id: "servers.backups.item.failed-to-prepare-backup",
    defaultMessage: "Failed to prepare download",
  },
  automated: {
    id: "servers.backups.item.automated",
    defaultMessage: "Automated",
  },
  retry: {
    id: "servers.backups.item.retry",
    defaultMessage: "Retry",
  },
});
</script>
<template>
  <div
    :class="
      preview
        ? 'grid-cols-[min-content_1fr_1fr] sm:grid-cols-[min-content_3fr_2fr_1fr] md:grid-cols-[auto_3fr_2fr_1fr]'
        : 'grid-cols-[min-content_1fr_1fr] sm:grid-cols-[min-content_3fr_2fr_1fr] md:grid-cols-[auto_3fr_2fr_1fr_2fr]'
    "
    class="grid items-center gap-4 rounded-2xl bg-bg-raised px-4 py-3"
  >
    <div
      class="flex h-12 w-12 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg"
    >
      <SpinnerIcon
        v-if="creating"
        class="h-6 w-6 animate-spin"
        :class="{ 'text-orange': backupQueued, 'text-green': !backupQueued }"
      />
      <FolderArchiveIcon v-else class="h-6 w-6" />
    </div>
    <div class="col-span-2 flex flex-col gap-1 sm:col-span-1">
      <span class="font-bold text-contrast">
        {{ backup.name }}
      </span>
      <div class="flex flex-wrap items-center gap-2 text-sm">
        <span v-if="backup.locked" class="flex items-center gap-1 text-sm text-secondary">
          <LockIcon /> {{ formatMessage(messages.locked) }}
        </span>
        <span v-if="automated && backup.locked">•</span>
        <span v-if="automated" class="flex items-center gap-1 text-secondary">
          <BotIcon /> {{ formatMessage(messages.automated) }}
        </span>
        <span v-if="(failedToCreate || failedToRestore) && (automated || backup.locked)">•</span>
        <span
          v-if="failedToCreate || failedToRestore || failedToPrepareFile"
          class="flex items-center gap-1 text-sm text-red"
        >
          <XIcon />
          {{
            formatMessage(
              failedToCreate
                ? messages.failedToCreateBackup
                : failedToRestore
                  ? messages.failedToRestoreBackup
                  : messages.failedToPrepareFile,
            )
          }}
        </span>
      </div>
    </div>
    <div v-if="creating" class="col-span-2 flex flex-col gap-3">
      <span v-if="backupQueued" class="text-orange">
        {{ formatMessage(messages.queuedForBackup) }}
      </span>
      <span v-else class="text-green"> {{ formatMessage(messages.creatingBackup) }} </span>
      <ProgressBar
        :progress="creating.progress"
        :color="backupQueued ? 'orange' : 'green'"
        :waiting="creating.progress === 0"
        class="max-w-full"
      />
    </div>
    <div v-else-if="restoring" class="col-span-2 flex flex-col gap-3 text-purple">
      {{ formatMessage(messages.restoringBackup) }}
      <ProgressBar
        :progress="restoring.progress"
        color="purple"
        :waiting="restoring.progress === 0"
        class="max-w-full"
      />
    </div>
    <template v-else>
      <div class="col-span-2">
        {{ dayjs(backup.created_at).format("MMMM D, YYYY [at] h:mm A") }}
      </div>
      <div v-if="false">{{ 245 }} MiB</div>
    </template>
    <div
      v-if="!preview"
      class="col-span-full flex justify-normal gap-2 md:col-span-1 md:justify-end"
    >
      <template v-if="failedToCreate">
        <ButtonStyled>
          <button @click="() => emit('retry')">
            <RotateCounterClockwiseIcon />
            {{ formatMessage(messages.retry) }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="() => emit('delete', true)">
            <TrashIcon />
            Remove
          </button>
        </ButtonStyled>
      </template>
      <ButtonStyled v-else-if="creating">
        <button @click="() => emit('delete')">
          <XIcon />
          {{ formatMessage(commonMessages.cancelButton) }}
        </button>
      </ButtonStyled>
      <template v-else>
        <ButtonStyled>
          <a
            v-if="hasPreparedDownload"
            :class="{
              disabled: !kyrosUrl || !jwt,
            }"
            :href="`https://${kyrosUrl}/modrinth/v0/backups/${backup.id}/download?auth=${jwt}`"
            @click="() => emit('download')"
          >
            <DownloadIcon />
            {{ formatMessage(commonMessages.downloadButton) }}
          </a>
          <button
            v-else
            :disabled="!!preparingFile"
            @click="
              () => {
                initiatedPrepare = true;
                emit('prepare');
              }
            "
          >
            <SpinnerIcon v-if="preparingFile" class="animate-spin" />
            <DownloadIcon v-else />
            {{
              formatMessage(
                preparingFile
                  ? messages.preparingDownload
                  : failedToPrepareFile
                    ? messages.prepareDownloadAgain
                    : messages.prepareDownload,
              )
            }}
          </button>
        </ButtonStyled>
        <ButtonStyled circular type="transparent">
          <OverflowMenu
            :options="[
              { id: 'rename', action: () => emit('rename') },
              {
                id: 'restore',
                action: () => emit('restore'),
                disabled: !!restoring || !!preparingFile,
              },
              { id: 'lock', action: () => emit('lock') },
              { divider: true },
              {
                id: 'delete',
                color: 'red',
                action: () => emit('delete'),
                disabled: !!restoring || !!preparingFile,
              },
            ]"
          >
            <MoreVerticalIcon />
            <template #rename> <EditIcon /> {{ formatMessage(messages.rename) }} </template>
            <template #restore> <HistoryIcon /> {{ formatMessage(messages.restore) }} </template>
            <template v-if="backup.locked" #lock>
              <LockOpenIcon /> {{ formatMessage(messages.unlock) }}
            </template>
            <template v-else #lock> <LockIcon /> {{ formatMessage(messages.lock) }} </template>
            <template #delete>
              <TrashIcon /> {{ formatMessage(commonMessages.deleteLabel) }}
            </template>
          </OverflowMenu>
        </ButtonStyled>
      </template>
    </div>
    <pre
      v-if="!preview && flags.advancedDebugInfo"
      class="col-span-full m-0 rounded-xl bg-button-bg text-xs"
      >{{ backup }}</pre
    >
  </div>
</template>
