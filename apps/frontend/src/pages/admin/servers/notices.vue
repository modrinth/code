<template>
  <NewModal ref="createNoticeModal">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">{{
        editingNotice ? `Editing notice #${editingNotice?.id}` : "Creating a notice"
      }}</span>
    </template>
    <div class="flex w-[700px] flex-col gap-3">
      <div class="flex items-center justify-between gap-2">
        <label for="level-selector" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Level </span>
          <span>Determines how the notice should be styled.</span>
        </label>
        <TeleportDropdownMenu
          id="level-selector"
          v-model="newNoticeLevel"
          class="max-w-[10rem]"
          :options="levelOptions"
          :display-name="(x) => formatMessage(x.name)"
          name="Level"
        />
      </div>
      <div v-if="!newNoticeSurvey" class="flex flex-col gap-2">
        <label for="notice-title" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Title </span>
        </label>
        <input
          id="notice-title"
          v-model="newNoticeTitle"
          placeholder="E.g. Maintenance"
          type="text"
          autocomplete="off"
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="notice-message" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            {{ newNoticeSurvey ? "Survey ID" : "Message" }}
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          v-if="newNoticeSurvey"
          id="notice-message"
          v-model="newNoticeMessage"
          placeholder="E.g. rXGtq2"
          type="text"
          autocomplete="off"
        />
        <div v-else class="textarea-wrapper h-32">
          <textarea id="notice-message" v-model="newNoticeMessage" />
        </div>
      </div>
      <div v-if="!newNoticeSurvey" class="flex items-center justify-between gap-2">
        <label for="dismissable-toggle" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Dismissable </span>
          <span>Allow users to dismiss the notice from their panel.</span>
        </label>
        <Toggle id="dismissable-toggle" v-model="newNoticeDismissable" />
      </div>
      <div class="flex items-center justify-between gap-2">
        <label for="scheduled-date" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Announcement date </span>
          <span>Leave blank for notice to be available immediately.</span>
        </label>
        <input
          id="scheduled-date"
          v-model="newNoticeScheduledDate"
          type="datetime-local"
          autocomplete="off"
        />
      </div>
      <div class="flex items-center justify-between gap-2">
        <label for="expiration-date" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Expiration date </span>
          <span>The notice will automatically be deleted after this date.</span>
        </label>
        <input
          id="expiration-date"
          v-model="newNoticeExpiresDate"
          type="datetime-local"
          autocomplete="off"
        />
      </div>

      <div v-if="!newNoticeSurvey" class="flex flex-col gap-2">
        <span class="text-lg font-semibold text-contrast"> Preview </span>
        <ServerNotice
          :level="newNoticeLevel.id"
          :message="
            !trimmedMessage || trimmedMessage.length < 1
              ? 'Type a message to begin previewing it.'
              : trimmedMessage
          "
          :dismissable="newNoticeDismissable"
          :title="trimmedTitle"
          preview
        />
      </div>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button v-if="editingNotice" :disabled="!!noticeSubmitError" @click="() => saveChanges()">
            <SaveIcon aria-hidden="true" />
            {{ formatMessage(commonMessages.saveChangesButton) }}
          </button>
          <button v-else :disabled="!!noticeSubmitError" @click="() => createNotice()">
            <PlusIcon aria-hidden="true" />
            {{ formatMessage(messages.createNotice) }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="createNoticeModal?.hide">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
  <AssignNoticeModal ref="assignNoticeModal" @close="refreshNotices" />
  <div class="page experimental-styles-within">
    <div
      class="mb-6 flex items-end justify-between border-0 border-b border-solid border-divider pb-4"
    >
      <h1 class="m-0 text-2xl">Servers notices</h1>
      <ButtonStyled color="brand">
        <button @click="openNewNoticeModal">
          <PlusIcon />
          {{ formatMessage(messages.createNotice) }}
        </button>
      </ButtonStyled>
    </div>
    <div>
      <div v-if="!notices || notices.length === 0">{{ formatMessage(messages.noNotices) }}</div>
      <div
        v-else
        class="grid grid-cols-[auto_auto_auto] gap-4 md:grid-cols-[min-content_auto_auto_auto_auto_min-content]"
      >
        <div class="col-span-full grid grid-cols-subgrid gap-4 px-4 font-bold text-contrast">
          <div>{{ formatMessage(messages.id) }}</div>
          <div>{{ formatMessage(messages.begins) }}</div>
          <div>{{ formatMessage(messages.expires) }}</div>
          <div class="hidden md:block">{{ formatMessage(messages.level) }}</div>
          <div class="hidden md:block">{{ formatMessage(messages.dismissable) }}</div>
          <div class="hidden md:block">{{ formatMessage(messages.actions) }}</div>
        </div>
        <div
          v-for="notice in notices"
          :key="`notice-${notice.id}`"
          class="col-span-full grid grid-cols-subgrid gap-4 rounded-2xl bg-bg-raised p-4"
        >
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
                '--_color': notice.dismissable ? 'var(--color-green)' : 'var(--color-red)',
                '--_bg-color': notice.dismissable ? 'var(--color-green-bg)' : 'var(--color-red-bg)',
              }"
            >
              <TagItem>
                {{
                  formatMessage(notice.dismissable ? messages.dismissable : messages.undismissable)
                }}
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
              <template v-if="notice.dismissed_by.length > 0">
                •
                <span> Dismissed by {{ notice.dismissed_by.length }} servers </span>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import {
  CopyCode,
  TagItem,
  ButtonStyled,
  ServerNotice,
  commonMessages,
  NewModal,
  TeleportDropdownMenu,
  Toggle,
  useRelativeTime,
} from "@modrinth/ui";
import { SettingsIcon, PlusIcon, SaveIcon, TrashIcon, EditIcon, XIcon } from "@modrinth/assets";
import dayjs from "dayjs";
import { useVIntl } from "@vintl/vintl";
import type { ServerNotice as ServerNoticeType } from "@modrinth/utils";
import { computed } from "vue";
import { NOTICE_LEVELS } from "@modrinth/ui/src/utils/notices.ts";
import { usePyroFetch } from "~/composables/pyroFetch.ts";
import AssignNoticeModal from "~/components/ui/servers/notice/AssignNoticeModal.vue";

const { formatMessage } = useVIntl();
const formatRelativeTime = useRelativeTime();

const app = useNuxtApp() as unknown as { $notify: any };

const notices = ref<ServerNoticeType[]>([]);
const createNoticeModal = ref<InstanceType<typeof NewModal>>();
const assignNoticeModal = ref<InstanceType<typeof AssignNoticeModal>>();

await refreshNotices();

async function refreshNotices() {
  await usePyroFetch("notices").then((res) => {
    notices.value = res as ServerNoticeType[];
    notices.value.sort((a, b) => {
      const dateDiff = dayjs(b.announce_at).diff(dayjs(a.announce_at));
      if (dateDiff === 0) {
        return b.id - a.id;
      }

      return dateDiff;
    });
  });
}

const levelOptions = Object.keys(NOTICE_LEVELS).map((x) => ({
  id: x,
  ...NOTICE_LEVELS[x],
}));

const DATE_TIME_FORMAT = "YYYY-MM-DDTHH:mm";

const newNoticeLevel = ref(levelOptions[0]);
const newNoticeDismissable = ref(false);
const newNoticeMessage = ref("");
const newNoticeScheduledDate = ref<string>();
const newNoticeTitle = ref<string>();
const newNoticeExpiresDate = ref<string>();

function openNewNoticeModal() {
  newNoticeLevel.value = levelOptions[0];
  newNoticeDismissable.value = false;
  newNoticeMessage.value = "";
  newNoticeScheduledDate.value = undefined;
  newNoticeExpiresDate.value = undefined;
  editingNotice.value = undefined;
  createNoticeModal.value?.show();
}

const editingNotice = ref<undefined | ServerNoticeType>();

function startEditing(notice: ServerNoticeType, assignments: boolean = false) {
  newNoticeLevel.value = levelOptions.find((x) => x.id === notice.level) ?? levelOptions[0];
  newNoticeDismissable.value = notice.dismissable;
  newNoticeMessage.value = notice.message;
  newNoticeTitle.value = notice.title;
  newNoticeScheduledDate.value = dayjs(notice.announce_at).format(DATE_TIME_FORMAT);
  newNoticeExpiresDate.value = notice.expires
    ? dayjs(notice.expires).format(DATE_TIME_FORMAT)
    : undefined;
  editingNotice.value = notice;
  if (assignments) {
    assignNoticeModal.value?.show?.(notice);
  } else {
    createNoticeModal.value?.show();
  }
}

async function deleteNotice(notice: ServerNoticeType) {
  await usePyroFetch(`notices/${notice.id}`, {
    method: "DELETE",
  })
    .then(() => {
      app.$notify({
        group: "main",
        title: `Successfully deleted notice #${notice.id}`,
        type: "success",
      });
    })
    .catch((err) => {
      app.$notify({
        group: "main",
        title: "Error deleting notice",
        text: err,
        type: "error",
      });
    });
  await refreshNotices();
}

const trimmedMessage = computed(() => newNoticeMessage.value?.trim());
const trimmedTitle = computed(() => newNoticeTitle.value?.trim());
const newNoticeSurvey = computed(() => newNoticeLevel.value.id === "survey");

const noticeSubmitError = computed(() => {
  let error: undefined | string;
  if (!trimmedMessage.value || trimmedMessage.value.length === 0) {
    error = "Notice message is required";
  }
  if (!newNoticeLevel.value) {
    error = "Notice level is required";
  }
  return error;
});

function validateSubmission(message: string) {
  if (noticeSubmitError.value) {
    addNotification({
      group: "main",
      title: message,
      text: noticeSubmitError.value,
      type: "error",
    });
    return false;
  }
  return true;
}

async function saveChanges() {
  if (!validateSubmission("Error saving notice")) {
    return;
  }

  await usePyroFetch(`notices/${editingNotice.value?.id}`, {
    method: "PATCH",
    body: {
      message: newNoticeMessage.value,
      title: newNoticeSurvey.value ? undefined : trimmedTitle.value,
      level: newNoticeLevel.value.id,
      dismissable: newNoticeSurvey.value ? true : newNoticeDismissable.value,
      announce_at: newNoticeScheduledDate.value
        ? dayjs(newNoticeScheduledDate.value).toISOString()
        : dayjs().toISOString(),
      expires: newNoticeExpiresDate.value
        ? dayjs(newNoticeExpiresDate.value).toISOString()
        : undefined,
    },
  }).catch((err) => {
    app.$notify({
      group: "main",
      title: "Error saving changes to notice",
      text: err,
      type: "error",
    });
  });
  await refreshNotices();
  createNoticeModal.value?.hide();
}

async function createNotice() {
  if (!validateSubmission("Error creating notice")) {
    return;
  }

  await usePyroFetch("notices", {
    method: "POST",
    body: {
      message: newNoticeMessage.value,
      title: newNoticeSurvey.value ? undefined : trimmedTitle.value,
      level: newNoticeLevel.value.id,
      dismissable: newNoticeSurvey.value ? true : newNoticeDismissable.value,
      announce_at: newNoticeScheduledDate.value
        ? dayjs(newNoticeScheduledDate.value).toISOString()
        : dayjs().toISOString(),
      expires: newNoticeExpiresDate.value
        ? dayjs(newNoticeExpiresDate.value).toISOString()
        : undefined,
    },
  }).catch((err) => {
    app.$notify({
      group: "main",
      title: "Error creating notice",
      text: err,
      type: "error",
    });
  });
  await refreshNotices();
  createNoticeModal.value?.hide();
}

const messages = defineMessages({
  createNotice: {
    id: "servers.notices.create-notice",
    defaultMessage: "Create notice",
  },
  noNotices: {
    id: "servers.notices.no-notices",
    defaultMessage: "No notices",
  },
  dismissable: {
    id: "servers.notice.dismissable",
    defaultMessage: "Dismissable",
  },
  undismissable: {
    id: "servers.notice.undismissable",
    defaultMessage: "Undismissable",
  },
  id: {
    id: "servers.notice.id",
    defaultMessage: "ID",
  },
  begins: {
    id: "servers.notice.begins",
    defaultMessage: "Begins",
  },
  expires: {
    id: "servers.notice.expires",
    defaultMessage: "Expires",
  },
  actions: {
    id: "servers.notice.actions",
    defaultMessage: "Actions",
  },
  level: {
    id: "servers.notice.level",
    defaultMessage: "Level",
  },
});
</script>
<style lang="scss" scoped>
.page {
  padding: 1rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 78.5rem;
}
</style>
