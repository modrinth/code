<template>
  <NewModal ref="mrpackModal" header="Uploading mrpack" :closable="!isLoading" @show="onShow">
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
            <Transition name="phrase-fade" mode="out-in">
              <span :key="currentPhrase" class="text-lg font-medium text-contrast">{{
                currentPhrase
              }}</span>
            </Transition>
            <div class="flex flex-col items-end">
              <span class="text-secondary">{{ Math.round(uploadProgress) }}%</span>
              <span class="text-xs text-secondary"
                >{{ formatBytes(uploadedBytes) }} / {{ formatBytes(totalBytes) }}</span
              >
            </div>
          </div>
          <div class="h-2 w-full rounded-full bg-divider">
            <div
              class="h-2 animate-pulse rounded-full bg-brand transition-all duration-300 ease-out"
              :style="{ width: `${uploadProgress}%` }"
            ></div>
          </div>
        </div>
      </Transition>

      <Transition
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 max-h-0"
        enter-to-class="opacity-100 max-h-20"
        leave-active-class="transition-all duration-200 ease-in"
        leave-from-class="opacity-100 max-h-20"
        leave-to-class="opacity-0 max-h-0"
      >
        <div v-if="!isLoading" class="flex flex-col gap-4">
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
              <ArrowBigRightDashIcon class="size-10" />
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
                Removes all data on your server, including your worlds, mods, and configuration
                files, then reinstalls it with the selected version.
              </div>
              <div class="font-bold">
                This does not affect your backups, which are stored off-site.
              </div>
            </div>

            <BackupWarning :backup-link="`/servers/manage/${props.server?.serverId}/backups`" />
          </div>
          <div class="mt-4 flex justify-start gap-4">
            <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
              <button
                v-tooltip="backupInProgress ? backupInProgress.tooltip : undefined"
                :disabled="canInstall || !!backupInProgress"
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
      </Transition>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { BackupWarning, ButtonStyled, NewModal } from "@modrinth/ui";
import {
  UploadIcon,
  RightArrowIcon,
  XIcon,
  ServerIcon,
  ArrowBigRightDashIcon,
} from "@modrinth/assets";
import { formatBytes, ModrinthServersFetchError } from "@modrinth/utils";
import { onMounted, onUnmounted } from "vue";
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";
import type { ModrinthServer } from "~/composables/servers/modrinth-servers";

const handleBeforeUnload = (event: BeforeUnloadEvent) => {
  if (isLoading.value) {
    event.preventDefault();
    return "Upload in progress. Are you sure you want to leave?";
  }
};

onMounted(() => {
  window.addEventListener("beforeunload", handleBeforeUnload);
});

onUnmounted(() => {
  window.removeEventListener("beforeunload", handleBeforeUnload);
});

const props = defineProps<{
  server: ModrinthServer;
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
const uploadedBytes = ref(0);
const totalBytes = ref(0);

const uploadPhrases = [
  "Removing Herobrine...",
  "Feeding parrots...",
  "Teaching villagers new trades...",
  "Convincing creepers to be friendly...",
  "Polishing diamonds...",
  "Training wolves to fetch...",
  "Building pixel art...",
  "Explaining redstone to beginners...",
  "Collecting all the cats...",
  "Negotiating with endermen...",
  "Planting suspicious stew ingredients...",
  "Calibrating TNT blast radius...",
  "Teaching chickens to fly...",
  "Sorting inventory alphabetically...",
  "Convincing iron golems to smile...",
];

const currentPhrase = ref("Uploading...");
let phraseInterval: NodeJS.Timeout | null = null;
const usedPhrases = ref(new Set<number>());

const getNextPhrase = () => {
  if (usedPhrases.value.size >= uploadPhrases.length) {
    const currentPhraseIndex = uploadPhrases.indexOf(currentPhrase.value);
    usedPhrases.value.clear();
    if (currentPhraseIndex !== -1) {
      usedPhrases.value.add(currentPhraseIndex);
    }
  }
  const availableIndices = uploadPhrases
    .map((_, index) => index)
    .filter((index) => !usedPhrases.value.has(index));

  const randomIndex = availableIndices[Math.floor(Math.random() * availableIndices.length)];
  usedPhrases.value.add(randomIndex);

  return uploadPhrases[randomIndex];
};

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
  uploadProgress.value = 0;
  uploadedBytes.value = 0;
  totalBytes.value = mrpackFile.value.size;

  currentPhrase.value = getNextPhrase();
  phraseInterval = setInterval(() => {
    currentPhrase.value = getNextPhrase();
  }, 4500);

  const { onProgress, promise } = props.server.general.reinstallFromMrpack(
    mrpackFile.value,
    hardReset.value,
  );

  onProgress(({ loaded, total, progress }) => {
    uploadProgress.value = progress;
    uploadedBytes.value = loaded;
    totalBytes.value = total;

    if (phraseInterval && progress >= 100) {
      clearInterval(phraseInterval);
      phraseInterval = null;
      currentPhrase.value = "Installing modpack...";
    }
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
    if (error instanceof ModrinthServersFetchError && error.statusCode === 429) {
      addNotification({
        group: "server",
        title: "Cannot upload and install modpack to server",
        text: "You are being rate limited. Please try again later.",
        type: "error",
      });
    } else {
      addNotification({
        group: "server",
        title: "Modpack upload and install failed",
        text: "An unexpected error occurred while uploading/installing. Please try again later.",
        type: "error",
      });
    }
  } finally {
    isLoading.value = false;
    if (phraseInterval) {
      clearInterval(phraseInterval);
      phraseInterval = null;
    }
  }
};
const onShow = () => {
  hardReset.value = false;
  isMrpackModalSecondPhase.value = false;
  loadingServerCheck.value = false;
  isLoading.value = false;
  mrpackFile.value = null;
  uploadProgress.value = 0;
  uploadedBytes.value = 0;
  totalBytes.value = 0;
  currentPhrase.value = "Uploading...";
  usedPhrases.value.clear();
  if (phraseInterval) {
    clearInterval(phraseInterval);
    phraseInterval = null;
  }
};

const show = () => mrpackModal.value?.show();
const hide = () => mrpackModal.value?.hide();

defineExpose({ show, hide });
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}

.phrase-fade-enter-active,
.phrase-fade-leave-active {
  transition: opacity 0.3s ease;
}

.phrase-fade-enter-from,
.phrase-fade-leave-to {
  opacity: 0;
}
</style>
