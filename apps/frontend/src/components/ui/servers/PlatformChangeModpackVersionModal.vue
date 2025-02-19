<template>
  <NewModal
    ref="modal"
    :header="'Changing ' + props.project?.title + ' version'"
    @hide="onHide"
    @show="onShow"
  >
    <div class="flex flex-col gap-4 md:w-[600px]">
      <div class="flex flex-col gap-2">
        <p class="m-0">
          Select the version of {{ props.project?.title || "the modpack" }} you want to install on
          your server.
        </p>
        <p v-if="props.currentVersion" class="m-0 text-sm text-secondary">
          Currently installed: {{ props.currentVersion.version_number }}
        </p>
      </div>

      <div class="flex w-full flex-col gap-4">
        <UiServersTeleportDropdownMenu
          v-if="props.versions?.length"
          v-model="selectedVersion"
          :options="versionOptions"
          placeholder="Select version..."
          name="version"
          class="w-full max-w-full"
        />

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="modpack-hard-reset">
              Erase all data
            </label>
            <input
              id="modpack-hard-reset"
              v-model="hardReset"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
            />
          </div>
          <div>
            If enabled, existing mods, worlds, and configurations, will be deleted before installing
            the new modpack version.
          </div>
        </div>

        <div class="mt-4 flex justify-start gap-4">
          <ButtonStyled :color="hardReset ? 'red' : 'brand'">
            <button
              :disabled="isLoading || !selectedVersion || props.serverStatus === 'installing'"
              @click="handleReinstall"
            >
              <DownloadIcon class="size-4" />
              {{ isLoading ? "Installing..." : hardReset ? "Erase and install" : "Install" }}
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button :disabled="isLoading" @click="hide">
              <XIcon />
              Cancel
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { DownloadIcon, XIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  project: any;
  versions: any[];
  currentVersion?: any;
  currentVersionId?: string;
  serverStatus?: string;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const modal = ref();
const hardReset = ref(false);
const isLoading = ref(false);
const selectedVersion = ref("");

const versionOptions = computed(() => props.versions?.map((v) => v.version_number) || []);

const handleReinstall = async () => {
  if (!selectedVersion.value || !props.project?.id) return;

  isLoading.value = true;
  try {
    const versionId = props.versions.find((v) => v.version_number === selectedVersion.value)?.id;

    await props.server.general?.reinstall(
      props.server.serverId,
      false,
      props.project.id,
      versionId,
      undefined,
      hardReset.value,
    );

    emit("reinstall");
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

watch(
  () => props.serverStatus,
  (newStatus) => {
    if (newStatus === "installing") {
      hide();
    }
  },
);

const onShow = () => {
  hardReset.value = false;
  selectedVersion.value =
    props.currentVersion?.version_number ?? props.versions?.[0]?.version_number ?? "";
};

const onHide = () => {
  hardReset.value = false;
  selectedVersion.value = "";
  isLoading.value = false;
};

const show = () => modal.value?.show();
const hide = () => modal.value?.hide();

defineExpose({ show, hide });
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
