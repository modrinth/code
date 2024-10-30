<template>
  <div class="contents">
    <NewModal ref="newAllocationModal" header="New Allocation">
      <form class="flex flex-col gap-2 md:w-[600px]" @submit.prevent="addNewAllocation">
        <label for="new-allocation-name" class="font-semibold text-contrast"> Name </label>
        <input
          id="new-allocation-name"
          ref="newAllocationInput"
          v-model="newAllocationName"
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          maxlength="32"
          placeholder="e.g. Secondary allocation"
        />
        <div class="mb-1 mt-4 flex justify-start gap-4">
          <ButtonStyled color="brand">
            <button :disabled="!newAllocationName" type="submit">
              <PlusIcon /> Create allocation
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button @click="newAllocationModal?.hide()">Cancel</button>
          </ButtonStyled>
        </div>
      </form>
    </NewModal>

    <NewModal ref="editAllocationModal" header="Edit Allocation">
      <form class="flex flex-col gap-2 md:w-[600px]" @submit.prevent="editAllocation">
        <label for="edit-allocation-name" class="font-semibold text-contrast"> Name </label>
        <input
          id="edit-allocation-name"
          ref="editAllocationInput"
          v-model="newAllocationName"
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          maxlength="32"
          placeholder="e.g. Secondary allocation"
        />
        <div class="mb-1 mt-4 flex justify-start gap-4">
          <ButtonStyled color="brand">
            <button :disabled="!newAllocationName" type="submit">
              <SaveIcon /> Update Allocation
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button @click="editAllocationModal?.hide()">Cancel</button>
          </ButtonStyled>
        </div>
      </form>
    </NewModal>

    <ConfirmModal
      ref="confirmDeleteModal"
      title="Are you sure you want to delete this allocation?"
      description="This will permanently delete this allocation."
      proceed-label="Delete"
      @proceed="confirmDeleteAllocation"
    />

    <div class="relative h-full w-full overflow-y-auto">
      <div v-if="data" class="flex h-full w-full flex-col justify-between gap-4">
        <div class="flex h-full flex-col">
          <!-- Subdomain section -->
          <div class="card flex flex-col gap-4">
            <label for="server-subdomain" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Custom subdomain</span>
              <span> Your friends can connect to your server using this URL. </span>
            </label>
            <div class="flex w-full items-center gap-2 md:w-[60%]">
              <input
                id="server-subdomain"
                v-model="serverSubdomain"
                class="h-[50%] w-[63%]"
                maxlength="32"
                @keyup.enter="saveNetwork"
              />
              .modrinth.gg
            </div>
            <span v-if="!isValidSubdomain" class="text-sm text-rose-400">
              Subdomain must be at least 5 characters long and can only contain alphanumeric
              characters and dashes.
            </span>
          </div>

          <div class="card flex flex-col gap-4">
            <div class="flex w-full flex-row items-center justify-between">
              <label for="user-domain" class="flex flex-col gap-2">
                <span class="text-lg font-bold text-contrast">Generated DNS records</span>
                <span>
                  Set up your personal domain to connect to your server via custom DNS records.
                </span>
              </label>

              <ButtonStyled>
                <button @click="exportDnsRecords">
                  <UploadIcon />
                  <span>Export DNS Records</span>
                </button>
              </ButtonStyled>
            </div>

            <input
              id="user-domain"
              v-model="userDomain"
              class="w-full md:w-[50%]"
              maxlength="64"
              minlength="1"
              type="text"
              placeholder="domain.com"
            />

            <div
              class="flex max-w-full flex-none overflow-auto rounded-xl bg-table-alternateRow p-4"
            >
              <table
                class="w-full flex-none border-collapse truncate rounded-lg border-2 border-gray-300"
              >
                <tbody class="w-full">
                  <tr v-for="record in dnsRecords" :key="record.content" class="w-full">
                    <td class="w-1/6 py-3 pr-4 md:w-1/5 md:pr-8 lg:w-1/4 lg:pr-12">
                      <div class="flex flex-col gap-1" @click="copyText(record.type)">
                        <span
                          class="text-md font-bold tracking-wide text-contrast hover:cursor-pointer"
                        >
                          {{ record.type }}
                        </span>
                        <span class="text-xs uppercase text-secondary">type</span>
                      </div>
                    </td>
                    <td class="w-2/6 py-3 md:w-1/3">
                      <div class="flex flex-col gap-1" @click="copyText(record.name)">
                        <span
                          class="text-md truncate font-bold tracking-wide text-contrast hover:cursor-pointer"
                        >
                          {{ record.name }}
                        </span>
                        <span class="text-xs uppercase text-secondary">name</span>
                      </div>
                    </td>
                    <td class="w-3/6 py-3 pl-4 md:w-5/12 lg:w-5/12">
                      <div class="flex flex-col gap-1" @click="copyText(record.content)">
                        <span
                          class="text-md w-fit truncate font-bold tracking-wide text-contrast hover:cursor-pointer"
                        >
                          {{ record.content }}
                        </span>
                        <span class="text-xs uppercase text-secondary">content</span>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div class="flex items-center gap-2">
              <InfoIcon />
              <span class="text-sm text-secondary">
                You must own your own domain to use this feature.
              </span>
            </div>
          </div>

          <!-- Allocations section -->
          <div class="card flex flex-col gap-4">
            <div class="flex w-full flex-row items-center justify-between">
              <div class="flex flex-col gap-2">
                <span class="text-lg font-bold text-contrast">Allocations</span>
                <span>
                  Configure additional ports for internet-facing features like map viewers or voice
                  chat mods.
                </span>
              </div>

              <ButtonStyled type="standard" color="brand" @click="showNewAllocationModal">
                <button>
                  <PlusIcon />
                  <span>New allocation</span>
                </button>
              </ButtonStyled>
            </div>

            <div class="flex w-full flex-col overflow-hidden rounded-xl bg-table-alternateRow p-4">
              <!-- Primary allocation -->
              <div class="flex flex-row items-center justify-between">
                <span class="text-md font-bold capitalize tracking-wide text-contrast">
                  Primary Allocation
                </span>

                <UiCopyCode :text="`${serverIP}:${serverPrimaryPort}`" />
              </div>
            </div>

            <div
              v-if="allocations?.[0]"
              class="flex w-full flex-col gap-4 overflow-hidden rounded-xl bg-table-alternateRow p-4"
            >
              <div
                v-for="allocation in allocations"
                :key="allocation.port"
                class="border-border flex flex-row items-center justify-between"
              >
                <div class="flex flex-row items-center gap-4">
                  <VersionIcon class="h-7 w-7 rotate-90" />
                  <div class="flex w-[20rem] flex-row items-center justify-between">
                    <div class="flex flex-col gap-1">
                      <span class="text-md font-bold capitalize tracking-wide text-contrast">
                        {{ allocation.name }}
                      </span>
                      <span class="text-xs uppercase text-secondary">name</span>
                    </div>
                    <div class="flex flex-col gap-1">
                      <span class="text-md w-10 font-bold capitalize tracking-wide text-contrast">
                        {{ allocation.port }}
                      </span>
                      <span class="text-xs uppercase text-secondary">port</span>
                    </div>
                  </div>
                </div>

                <div class="flex flex-row items-center gap-2">
                  <ButtonStyled icon-only>
                    <button @click="showEditAllocationModal(allocation.port)">
                      <EditIcon />
                    </button>
                  </ButtonStyled>
                  <ButtonStyled icon-only color="red">
                    <button @click="showConfirmDeleteModal(allocation.port)">
                      <TrashIcon />
                    </button>
                  </ButtonStyled>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <UiServersSaveBanner
        :is-visible="!!hasUnsavedChanges && !!isValidSubdomain"
        :server="props.server"
        :is-updating="isUpdating"
        :save="saveNetwork"
        :reset="resetNetwork"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  PlusIcon,
  TrashIcon,
  EditIcon,
  VersionIcon,
  SaveIcon,
  InfoIcon,
  UploadIcon,
} from "@modrinth/assets";
import { ButtonStyled, NewModal, ConfirmModal } from "@modrinth/ui";
import { ref, computed, nextTick } from "vue";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const isUpdating = ref(false);
const data = computed(() => props.server.general);

const serverIP = ref(data?.value?.net?.ip ?? "");
const serverSubdomain = ref(data?.value?.net?.domain ?? "");
const serverPrimaryPort = ref(data?.value?.net?.port ?? 0);
const userDomain = ref("play.yourdomain.com");

const network = computed(() => props.server.network);
const allocations = computed(() => network.value?.allocations);

const newAllocationModal = ref<typeof NewModal>();
const editAllocationModal = ref<typeof NewModal>();
const confirmDeleteModal = ref<typeof ConfirmModal>();
const newAllocationInput = ref<HTMLInputElement | null>(null);
const editAllocationInput = ref<HTMLInputElement | null>(null);
const newAllocationName = ref("");
const newAllocationPort = ref(0);
const allocationToDelete = ref<number | null>(null);

const hasUnsavedChanges = computed(() => serverSubdomain.value !== data?.value?.net?.domain);

const isValidSubdomain = computed(() => /^[a-zA-Z0-9-]{5,}$/.test(serverSubdomain.value));

const addNewAllocation = async () => {
  if (!newAllocationName.value) return;

  try {
    await props.server.network?.reserveAllocation(newAllocationName.value);

    newAllocationModal.value?.hide();
    newAllocationName.value = "";

    await props.server.refresh();

    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Allocation reserved",
      text: "Your allocation has been reserved.",
    });
  } catch (error) {
    console.error("Failed to reserve new allocation:", error);
  }
};

const showNewAllocationModal = () => {
  newAllocationName.value = "";
  newAllocationModal.value?.show();
  nextTick(() => {
    setTimeout(() => {
      newAllocationInput.value?.focus();
    }, 100);
  });
};

const showEditAllocationModal = (port: number) => {
  newAllocationPort.value = port;
  editAllocationModal.value?.show();
  nextTick(() => {
    setTimeout(() => {
      editAllocationInput.value?.focus();
    }, 100);
  });
};

const showConfirmDeleteModal = (port: number) => {
  allocationToDelete.value = port;
  confirmDeleteModal.value?.show();
};

const confirmDeleteAllocation = async () => {
  if (allocationToDelete.value === null) return;

  await props.server.network?.deleteAllocation(allocationToDelete.value);

  await props.server.refresh();
  addNotification({
    group: "serverOptions",
    type: "success",
    title: "Allocation removed",
    text: "Your allocation has been removed.",
  });

  allocationToDelete.value = null;
};

const editAllocation = async () => {
  if (!newAllocationName.value) return;

  try {
    await props.server.network?.updateAllocation(newAllocationPort.value, newAllocationName.value);

    editAllocationModal.value?.hide();
    newAllocationName.value = "";

    await props.server.refresh();

    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Allocation updated",
      text: "Your allocation has been updated.",
    });
  } catch (error) {
    console.error("Failed to reserve new allocation:", error);
  }
};

const saveNetwork = async () => {
  if (!isValidSubdomain.value) return;

  try {
    isUpdating.value = true;
    const available = await props.server.network?.checkSubdomainAvailability(serverSubdomain.value);
    if (!available) {
      addNotification({
        group: "serverOptions",
        type: "error",
        title: "Subdomain not available",
        text: "The subdomain you entered is already in use.",
      });
      return;
    }
    if (serverSubdomain.value !== data?.value?.net?.domain) {
      await props.server.network?.changeSubdomain(serverSubdomain.value);
    }
    if (serverPrimaryPort.value !== data?.value?.net?.port) {
      await props.server.network?.updateAllocation(
        serverPrimaryPort.value,
        newAllocationName.value,
      );
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
    await props.server.refresh();
    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    console.error(error);
    addNotification({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server settings",
      text: "An error occurred while attempting to update your server settings.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetNetwork = () => {
  serverSubdomain.value = data?.value?.net?.domain ?? "";
};

const dnsRecords = computed(() => {
  return [
    {
      type: "A",
      name: `${userDomain.value}`,
      content: data.value?.net?.ip ?? "",
    },
    {
      type: "SRV",
      name: `_minecraft._tcp.${userDomain.value}`,
      content: `0 10 ${data.value?.net?.port} ${userDomain.value}`,
    },
  ];
});

const exportDnsRecords = () => {
  const records = dnsRecords.value.reduce(
    (acc, record) => {
      const type = record.type;
      if (!acc[type]) {
        acc[type] = [];
      }
      acc[type].push(record);
      return acc;
    },
    {} as Record<string, any[]>,
  );

  const text = Object.entries(records)
    .map(([type, records]) => {
      return `; ${type} Records\n${records.map((record) => `${record.name}.	1	IN	${record.type} ${record.content}${record.type === "SRV" ? "." : ""}`).join("\n")}\n`;
    })
    .join("\n");
  const blob = new Blob([text], { type: "text/plain" });
  const a = document.createElement("a");
  a.href = window.URL.createObjectURL(blob);
  a.download = `${userDomain.value}.txt`;
  a.click();
  a.remove();
};

const copyText = (text: string) => {
  navigator.clipboard.writeText(text);
  addNotification({
    group: "serverOptions",
    type: "success",
    title: "Text copied",
    text: `${text} has been copied to your clipboard`,
  });
};
</script>
