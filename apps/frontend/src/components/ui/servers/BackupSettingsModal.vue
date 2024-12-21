<template>
  <NewModal ref="modal" header="Editing auto backup settings">
    <div class="flex flex-col gap-4 md:w-[600px]">
      <div class="flex flex-col gap-2">
        <div class="font-semibold text-contrast">Auto backup</div>
        <p class="m-0">
          Automatically create a backup of your server every
          <strong>{{ autoBackupInterval == 1 ? "hour" : `${autoBackupInterval} hours` }}</strong>
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
            The amount of hours between each backup. This will only backup your server if it has
            been modified since the last backup.
          </p>
        </div>

        <div class="flex items-center gap-2 text-contrast">
          <div
            class="flex w-fit items-center rounded-xl border border-solid border-button-border bg-table-alternateRow"
          >
            <button
              class="rounded-l-xl p-3 text-secondary enabled:hover:text-contrast [&&]:bg-transparent enabled:[&&]:hover:bg-button-bg"
              :disabled="!autoBackupEnabled || isSaving"
              @click="autoBackupInterval = Math.max(autoBackupInterval - 1, 1)"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="2" viewBox="-2 0 18 2">
                <path
                  d="M18,12H6"
                  transform="translate(-5 -11)"
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                />
              </svg>
            </button>
            <input
              id="auto-backup-interval"
              v-model="autoBackupInterval"
              class="w-16 !appearance-none text-center [&&]:bg-transparent [&&]:focus:shadow-none"
              type="number"
              style="-moz-appearance: textfield; appearance: none"
              min="1"
              max="24"
              step="1"
              :disabled="!autoBackupEnabled || isSaving"
            />

            <button
              class="rounded-r-xl p-3 text-secondary enabled:hover:text-contrast [&&]:bg-transparent enabled:[&&]:hover:bg-button-bg"
              :disabled="!autoBackupEnabled || isSaving"
              @click="autoBackupInterval = Math.min(autoBackupInterval + 1, 24)"
            >
              <PlusIcon />
            </button>
          </div>
          {{ autoBackupInterval == 1 ? "hour" : "hours" }}
        </div>

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
import { PlusIcon, XIcon, SaveIcon } from "@modrinth/assets";
import { ref, computed } from "vue";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["backups"]>;
}>();

const modal = ref<InstanceType<typeof NewModal>>();

const initialSettings = ref<{ interval: number; enabled: boolean } | null>(null);
const autoBackupEnabled = ref(false);
const autoBackupInterval = ref(6);
const isLoadingSettings = ref(true);
const isSaving = ref(false);

const validatedBackupInterval = computed(() => {
  const roundedValue = Math.round(autoBackupInterval.value);

  if (roundedValue < 1) {
    return 1;
  } else if (roundedValue > 24) {
    return 24;
  }
  return roundedValue;
});

const hasChanges = computed(() => {
  if (!initialSettings.value) return false;

  return (
    autoBackupEnabled.value !== initialSettings.value.enabled ||
    autoBackupInterval.value !== initialSettings.value.interval
  );
});

const fetchSettings = async () => {
  isLoadingSettings.value = true;
  try {
    const settings = await props.server.backups?.getAutoBackup();
    initialSettings.value = settings as { interval: number; enabled: boolean };
    autoBackupEnabled.value = settings?.enabled ?? false;
    autoBackupInterval.value = settings?.interval || 6;
  } catch (error) {
    console.error("Error fetching backup settings:", error);
    addNotification({
      group: "server",
      title: "Error",
      text: "Failed to load backup settings",
      type: "error",
    });
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

watch(autoBackupInterval, () => {
  autoBackupInterval.value = validatedBackupInterval.value;
});

defineExpose({
  show: async () => {
    await fetchSettings();
    modal.value?.show();
  },
});
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
