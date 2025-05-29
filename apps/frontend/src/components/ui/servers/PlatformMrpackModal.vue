<template>
  <NewModal ref="mrpackModal" header="Uploading mrpack" @hide="onHide" @show="onShow">
    <div class="flex flex-col gap-4 md:w-[600px]">
      <Transition
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 max-h-0"
        enter-to-class="opacity-100 max-h-20"
        leave-active-class="transition-all duration-200 ease-in"
        leave-from-class="opacity-100 max-h-20"
        leave-to-class="opacity-0 max-h-0"
      >
        <div v-if="isLoading" class="w-full">
          <div class="mb-2 flex justify-between text-sm">
            <span class="font-medium text-contrast">Uploading...</span>
            <span class="text-secondary">{{ Math.round(uploadProgress) }}%</span>
          </div>
          <div class="h-2 w-full rounded-full bg-divider">
            <div
              class="h-2 rounded-full bg-brand transition-all duration-300 ease-out"
              :style="{ width: `${uploadProgress}%` }"
            ></div>
          </div>
        </div>
      </Transition>
      <p
        v-if="isMrpackModalSecondPhase"
        :style="{
          lineHeight: isMrpackModalSecondPhase ? '1.5' : undefined,
          marginBottom: isMrpackModalSecondPhase ? '-12px' : '0',
          marginTop: isMrpackModalSecondPhase ? '-4px' : '-2px',
        }"
      >
        This will reinstall your server and erase all data. You may want to back up your server
        before proceeding. Are you sure you want to continue?
      </p>
      <div v-if="!isMrpackModalSecondPhase" class="flex flex-col gap-4">
        <div class="mx-auto flex flex-row items-center gap-4">
          <div
            class="grid size-16 place-content-center rounded-2xl border-[2px] border-solid border-button-border bg-button-bg shadow-sm"
          >
            <UploadIcon class="size-10" />
          </div>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="size-10"
          >
            <path d="M5 9v6" />
            <path d="M9 9h3V5l7 7-7 7v-4H9V9z" />
          </svg>
          <div
            class="grid size-16 place-content-center rounded-2xl border-[2px] border-solid border-button-border bg-table-alternateRow shadow-sm"
          >
            <ServerIcon class="size-10" />
          </div>
        </div>
        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="text-sm font-bold text-contrast">Upload mrpack</div>
          <input
            type="file"
            accept=".mrpack"
            class=""
            :disabled="isLoading"
            @change="uploadMrpack"
          />
        </div>

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="hard-reset">
              Erase all data
            </label>
            <input
              id="hard-reset"
              v-model="hardReset"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
            />
          </div>
          <div>
            Removes all data on your server, including your worlds, mods, and configuration files,
            then reinstalls it with the selected version.
          </div>
          <div class="font-bold">This does not affect your backups, which are stored off-site.</div>
        </div>

        <BackupWarning :backup-link="`/servers/manage/${props.server?.serverId}/backups`" />
      </div>
      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button
            v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
            :disabled="canInstall || backupInProgress"
            @click="handleReinstall"
          >
            <RightArrowIcon />
            {{
              isMrpackModalSecondPhase
                ? "Erase and install"
                : loadingServerCheck
                  ? "Loading..."
                  : isDangerous
                    ? "Erase and install"
                    : "Install"
            }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button
            :disabled="isLoading"
            @click="
              () => {
                if (isMrpackModalSecondPhase) {
                  isMrpackModalSecondPhase = false;
                } else {
                  hide();
                }
              }
            "
          >
            <XIcon />
            {{ isMrpackModalSecondPhase ? "Go back" : "Cancel" }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { BackupWarning, ButtonStyled, NewModal } from "@modrinth/ui";
import { UploadIcon, RightArrowIcon, XIcon, ServerIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  backupInProgress?: BackupInProgressReason;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const mrpackModal = ref();
const isMrpackModalSecondPhase = ref(false);
const hardReset = ref(false);
const isLoading = ref(false);
const loadingServerCheck = ref(false);
const mrpackFile = ref<File | null>(null);
const uploadProgress = ref(0);

const isDangerous = computed(() => hardReset.value);
const canInstall = computed(() => !mrpackFile.value || isLoading.value || loadingServerCheck.value);

const uploadMrpack = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) {
    return;
  }
  mrpackFile.value = target.files[0];
};

const handleReinstall = async () => {
  if (hardReset.value && !isMrpackModalSecondPhase.value) {
    isMrpackModalSecondPhase.value = true;
    return;
  }

  if (!mrpackFile.value) {
    addNotification({
      group: "server",
      title: "No file selected",
      text: "Choose a .mrpack file before installing.",
      type: "error",
    });
    return;
  }

  isLoading.value = true;
  uploadProgress.value = 0;

  const { onProgress, promise } = props.server.general!.reinstallFromMrpack(
    mrpackFile.value,
    hardReset.value,
  );

  onProgress(({ progress }) => {
    uploadProgress.value = progress;
  });

  try {
    await promise;

    emit("reinstall", {
      loader: "mrpack",
      lVersion: "",
      mVersion: "",
    });

    await nextTick();
    window.scrollTo(0, 0);
    hide();
  } catch (error) {
    if (error instanceof PyroFetchError && error.statusCode === 429) {
      addNotification({
        group: "server",
        title: "Cannot reinstall server",
        text: "You are being rate limited. Please try again later.",
        type: "error",
      });
    } else {
      addNotification({
        group: "server",
        title: "Reinstall failed",
        text: "An unexpected error occurred while reinstalling. Please try again later.",
        type: "error",
      });
    }
  } finally {
    isLoading.value = false;
  }
};
const onShow = () => {
  hardReset.value = false;
  isMrpackModalSecondPhase.value = false;
  loadingServerCheck.value = false;
  isLoading.value = false;
  mrpackFile.value = null;
};

const onHide = () => {
  onShow();
};

const show = () => mrpackModal.value?.show();
const hide = () => mrpackModal.value?.hide();

defineExpose({ show, hide });
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
