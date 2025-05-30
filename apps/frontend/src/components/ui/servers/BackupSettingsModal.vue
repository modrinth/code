<template>
  <NewModal ref="modal" header="Editing auto backup settings">
    <div class="flex flex-col gap-4 md:w-[600px]">
      <div class="flex flex-col gap-2">
        <div class="font-semibold text-contrast">Auto backup</div>
        <p class="m-0">
          Automatically create a backup of your server
          <strong>{{ backupIntervalsLabel.toLowerCase() }}</strong>
        </p>
      </div>

      <div v-if="isLoadingSettings" class="py-2 text-sm text-secondary">Loading settings...</div>
      <template v-else>
        <input
          id="auto-backup-toggle"
          v-model="autoBackupEnabled"
          class="switch stylized-toggle"
          type="checkbox"
          :disabled="isSaving"
        />

        <div class="flex flex-col gap-2">
          <div class="font-semibold text-contrast">Interval</div>
          <p class="m-0">
            The amount of time between each backup. This will only backup your server if it has been
            modified since the last backup.
          </p>
        </div>

        <UiServersTeleportDropdownMenu
          :id="'interval-field'"
          v-model="backupIntervalsLabel"
          :disabled="!autoBackupEnabled || isSaving"
          name="interval"
          :options="Object.keys(backupIntervals)"
          placeholder="Backup interval"
        />

        <div class="mt-4 flex justify-start gap-4">
          <ButtonStyled color="brand">
            <button :disabled="!hasChanges || isSaving" @click="saveSettings">
              <SaveIcon class="h-5 w-5" />
              {{ isSaving ? "Saving..." : "Save changes" }}
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button :disabled="isSaving" @click="modal?.hide()">
              <XIcon />
              Cancel
            </button>
          </ButtonStyled>
        </div>
      </template>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { XIcon, SaveIcon } from "@modrinth/assets";
import { ref, computed } from "vue";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["backups"]>;
}>();

const modal = ref<InstanceType<typeof NewModal>>();

const initialSettings = ref<{ interval: number; enabled: boolean } | null>(null);
const autoBackupEnabled = ref(false);
const isLoadingSettings = ref(true);
const isSaving = ref(false);

const backupIntervals = {
  "Every 3 hours": 3,
  "Every 6 hours": 6,
  "Every 12 hours": 12,
  Daily: 24,
};

const backupIntervalsLabel = ref<keyof typeof backupIntervals>("Every 6 hours");

const autoBackupInterval = computed({
  get: () => backupIntervals[backupIntervalsLabel.value],
  set: (value) => {
    const [label] =
      Object.entries(backupIntervals).find(([_, interval]) => interval === value) || [];
    if (label) backupIntervalsLabel.value = label as keyof typeof backupIntervals;
  },
});

const hasChanges = computed(() => {
  if (!initialSettings.value) return false;

  return (
    autoBackupEnabled.value !== initialSettings.value.enabled ||
    (initialSettings.value.enabled && autoBackupInterval.value !== initialSettings.value.interval)
  );
});

const fetchSettings = async () => {
  isLoadingSettings.value = true;
  try {
    const settings = await props.server.backups?.getAutoBackup();
    initialSettings.value = settings as { interval: number; enabled: boolean };
    autoBackupEnabled.value = settings?.enabled ?? false;
    autoBackupInterval.value = settings?.interval || 6;
    return true;
  } catch (error) {
    console.error("Error fetching backup settings:", error);
    addNotification({
      group: "server",
      title: "Error",
      text: "Failed to load backup settings",
      type: "error",
    });
    return false;
  } finally {
    isLoadingSettings.value = false;
  }
};

const saveSettings = async () => {
  isSaving.value = true;
  try {
    await props.server.backups?.updateAutoBackup(
      autoBackupEnabled.value ? "enable" : "disable",
      autoBackupInterval.value,
    );

    initialSettings.value = {
      enabled: autoBackupEnabled.value,
      interval: autoBackupInterval.value,
    };

    addNotification({
      group: "server",
      title: "Success",
      text: "Backup settings updated successfully",
      type: "success",
    });

    modal.value?.hide();
  } catch (error) {
    console.error("Error saving backup settings:", error);
    addNotification({
      group: "server",
      title: "Error",
      text: "Failed to save backup settings",
      type: "error",
    });
  } finally {
    isSaving.value = false;
  }
};

defineExpose({
  show: async () => {
    const success = await fetchSettings();
    if (success) {
      modal.value?.show();
    }
  },
});
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
