<template>
  <NewModal ref="mrpackModal" header="Uploading mrpack" @hide="onHide" @show="onShow">
    <div class="flex flex-col gap-4 md:w-[600px]">
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
        </div>

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="backup-server-mrpack">
              Backup server
            </label>
            <input
              id="backup-server-mrpack"
              v-model="backupServer"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
            />
          </div>
          <div>Creates a backup of your server before proceeding.</div>
        </div>
      </div>
      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button :disabled="canInstall" @click="handleReinstall">
            <RightArrowIcon />
            {{
              isBackingUp
                ? "Backing up..."
                : isMrpackModalSecondPhase
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
              if (isMrpackModalSecondPhase) {
                isMrpackModalSecondPhase = false;
              } else {
                hide();
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
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { UploadIcon, RightArrowIcon, XIcon, ServerIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const mrpackModal = ref();
const isMrpackModalSecondPhase = ref(false);
const hardReset = ref(false);
const backupServer = ref(false);
const isLoading = ref(false);
const isBackingUp = ref(false);
const loadingServerCheck = ref(false);
const mrpackFile = ref<File | null>(null);

const isDangerous = computed(() => hardReset.value);
const canInstall = computed(() => !mrpackFile.value || isLoading.value || loadingServerCheck.value);

const uploadMrpack = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) {
    return;
  }
  mrpackFile.value = target.files[0];
};

const performBackup = async (): Promise<boolean> => {
  try {
    const date = new Date();
    const format = date.toLocaleString(navigator.language || "en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "numeric",
      second: "numeric",
      timeZoneName: "short",
    });
    const backupName = `Reinstallation - ${format}`;
    isLoading.value = true;
    const backupId = await props.server.backups?.create(backupName);
    isBackingUp.value = true;
    let attempts = 0;
    while (true) {
      attempts++;
      if (attempts > 100) {
        addNotification({
          group: "server",
          title: "Backup Failed",
          text: "An unexpected error occurred while backing up. Please try again later.",
        });
        return false;
      }
      await props.server.refresh(["backups"]);
      const backups = await props.server.backups?.data;
      const backup = backupId ? backups?.find((x) => x.id === backupId) : undefined;
      if (backup && !backup.ongoing) {
        isBackingUp.value = false;
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 5000));
    }
    return true;
  } catch {
    addNotification({
      group: "server",
      title: "Backup Failed",
      text: "An unexpected error occurred while backing up. Please try again later.",
    });
    return false;
  }
};

const handleReinstall = async () => {
  if (hardReset.value && !backupServer.value && !isMrpackModalSecondPhase.value) {
    isMrpackModalSecondPhase.value = true;
    return;
  }

  if (backupServer.value && !(await performBackup())) {
    isLoading.value = false;
    return;
  }

  isLoading.value = true;

  try {
    if (!mrpackFile.value) {
      throw new Error("No mrpack file selected");
    }

    const mrpack = new File([mrpackFile.value], mrpackFile.value.name, {
      type: mrpackFile.value.type,
    });

    await props.server.general?.reinstallFromMrpack(mrpack, hardReset.value);

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
        title: "Reinstall Failed",
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
  backupServer.value = false;
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
