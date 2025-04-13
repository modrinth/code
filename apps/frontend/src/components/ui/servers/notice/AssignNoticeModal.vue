<script setup lang="ts">
import { Accordion, ButtonStyled, NewModal, ServerNotice, TagItem } from "@modrinth/ui";
import { PlusIcon, XIcon } from "@modrinth/assets";
import type { ServerNotice as ServerNoticeType } from "@modrinth/utils";
import { ref } from "vue";
import { usePyroFetch } from "~/composables/pyroFetch.ts";

const app = useNuxtApp() as unknown as { $notify: any };

const modal = ref<InstanceType<typeof NewModal>>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const notice = ref<ServerNoticeType>();

const assigned = ref<ServerNoticeType["assigned"]>([]);

const assignedServers = computed(() => assigned.value.filter((n) => n.kind === "server") ?? []);
const assignedNodes = computed(() => assigned.value.filter((n) => n.kind === "node") ?? []);

const inputField = ref("");

async function refresh() {
  await usePyroFetch("notices").then((res) => {
    const notices = res as ServerNoticeType[];
    assigned.value = notices.find((n) => n.id === notice.value?.id)?.assigned ?? [];
  });
}

async function assign(server: boolean = true) {
  const input = inputField.value.trim();

  if (input !== "" && notice.value) {
    await usePyroFetch(`notices/${notice.value.id}/assign?${server ? "server" : "node"}=${input}`, {
      method: "PUT",
    }).catch((err) => {
      app.$notify({
        group: "main",
        title: "Error assigning notice",
        text: err,
        type: "error",
      });
    });
  } else {
    app.$notify({
      group: "main",
      title: "Error assigning notice",
      text: "No server or node specified",
      type: "error",
    });
  }
  await refresh();
}

async function unassignDetect() {
  const input = inputField.value.trim();

  const server = assignedServers.value.some((assigned) => assigned.id === input);
  const node = assignedNodes.value.some((assigned) => assigned.id === input);

  if (!server && !node) {
    app.$notify({
      group: "main",
      title: "Error unassigning notice",
      text: "ID is not an assigned server or node",
      type: "error",
    });
    return;
  }

  await unassign(input, server);
}

async function unassign(id: string, server: boolean = true) {
  if (notice.value) {
    await usePyroFetch(`notices/${notice.value.id}/unassign?${server ? "server" : "node"}=${id}`, {
      method: "PUT",
    }).catch((err) => {
      app.$notify({
        group: "main",
        title: "Error unassigning notice",
        text: err,
        type: "error",
      });
    });
  }
  await refresh();
}

function show(currentNotice: ServerNoticeType) {
  notice.value = currentNotice;
  assigned.value = currentNotice?.assigned ?? [];
  modal.value?.show();
}

function hide() {
  modal.value?.hide();
}

defineExpose({ show, hide });
</script>
<template>
  <NewModal ref="modal" :on-hide="() => emit('close')">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">
        Editing assignments of notice #{{ notice?.id }}
      </span>
    </template>
    <div class="flex flex-col gap-4">
      <ServerNotice
        v-if="notice"
        :level="notice.level"
        :message="notice.message"
        :dismissable="notice.dismissable"
        :title="notice.title"
        preview
      />
      <div class="flex flex-col gap-2">
        <label for="server-assign-field" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Assigned servers </span>
        </label>
        <Accordion
          v-if="assignedServers.length > 0"
          class="mb-2"
          open-by-default
          button-class="text-primary m-0 p-0 border-none bg-transparent active:scale-95"
        >
          <template #title> {{ assignedServers.length }} servers </template>
          <div class="mt-2 flex flex-wrap gap-2">
            <TagItem
              v-for="server in assignedServers"
              :key="`server-${server.id}`"
              :action="() => unassign(server.id, true)"
            >
              <XIcon />
              {{ server.id }}
            </TagItem>
          </div>
        </Accordion>
        <span v-else class="mb-2"> No servers assigned yet </span>

        <div class="flex flex-col gap-2">
          <label for="server-assign-field" class="flex flex-col gap-1">
            <span class="text-lg font-semibold text-contrast"> Assigned nodes </span>
          </label>
          <Accordion
            v-if="assignedNodes.length > 0"
            class="mb-2"
            open-by-default
            button-class="text-primary m-0 p-0 border-none bg-transparent active:scale-95"
          >
            <template #title> {{ assignedNodes.length }} nodes </template>
            <div class="mt-2 flex flex-wrap gap-2">
              <TagItem
                v-for="node in assignedNodes"
                :key="`node-${node.id}`"
                :action="
                  () => {
                    unassign(node.id, false);
                  }
                "
              >
                <XIcon />
                {{ node.id }}
              </TagItem>
            </div>
          </Accordion>
          <span v-else class="mb-2"> No nodes assigned yet </span>
        </div>
        <div class="flex w-[45rem] items-center gap-2">
          <input
            id="server-assign-field"
            v-model="inputField"
            class="w-full"
            type="text"
            autocomplete="off"
          />
          <ButtonStyled color="green" color-fill="text">
            <button class="shrink-0" @click="() => assign(true)">
              <PlusIcon />
              Add server
            </button>
          </ButtonStyled>
          <ButtonStyled color="blue" color-fill="text">
            <button class="shrink-0" @click="() => assign(false)">
              <PlusIcon />
              Add node
            </button>
          </ButtonStyled>
          <ButtonStyled color="red" color-fill="text">
            <button class="shrink-0" @click="() => unassignDetect()">
              <XIcon />
              Remove
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </NewModal>
</template>
