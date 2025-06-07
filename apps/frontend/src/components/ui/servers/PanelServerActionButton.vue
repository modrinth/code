<template>
  <div class="contents">
    <NewModal ref="confirmActionModal" header="Confirming power action" @close="resetPowerAction">
      <div class="flex flex-col gap-4 md:w-[400px]">
        <p class="m-0">
          Are you sure you want to <span class="lowercase">{{ confirmActionText }}</span> the
          server?
        </p>
        <UiCheckbox
          v-model="dontAskAgain"
          label="Don't ask me again"
          class="text-sm"
          :disabled="!powerAction"
        />
        <div class="flex flex-row gap-4">
          <ButtonStyled type="standard" color="brand" @click="executePowerAction">
            <button>
              <CheckIcon class="h-5 w-5" />
              {{ confirmActionText }} server
            </button>
          </ButtonStyled>
          <ButtonStyled @click="resetPowerAction">
            <button>
              <XIcon class="h-5 w-5" />
              Cancel
            </button>
          </ButtonStyled>
        </div>
      </div>
    </NewModal>

    <NewModal
      ref="detailsModal"
      :header="`All of ${serverName || 'Server'} info`"
      @close="closeDetailsModal"
    >
      <UiServersServerInfoLabels
        :server-data="serverData"
        :show-game-label="true"
        :show-loader-label="true"
        :uptime-seconds="uptimeSeconds"
        :column="true"
        class="mb-6 flex flex-col gap-2"
      />
      <div v-if="flags.advancedDebugInfo" class="markdown-body">
        <pre>{{ serverData }}</pre>
      </div>
      <ButtonStyled type="standard" color="brand" @click="closeDetailsModal">
        <button class="w-full">Close</button>
      </ButtonStyled>
    </NewModal>

    <div class="flex flex-row items-center gap-2 rounded-lg">
      <ButtonStyled v-if="isInstalling" type="standard" color="brand">
        <button disabled class="flex-shrink-0">
          <UiServersPanelSpinner class="size-5" /> Installing...
        </button>
      </ButtonStyled>

      <template v-else>
        <ButtonStyled v-if="showStopButton" type="transparent">
          <button :disabled="!canTakeAction" @click="initiateAction('Stop')">
            <div class="flex gap-1">
              <StopCircleIcon class="h-5 w-5" />
              <span>{{ isStoppingState ? "Stopping..." : "Stop" }}</span>
            </div>
          </button>
        </ButtonStyled>

        <ButtonStyled type="standard" color="brand">
          <button :disabled="!canTakeAction" @click="handlePrimaryAction">
            <div v-if="isTransitionState" class="grid place-content-center">
              <UiServersIconsLoadingIcon />
            </div>
            <component :is="isRunning ? UpdatedIcon : PlayIcon" v-else />
            <span>{{ primaryActionText }}</span>
          </button>
        </ButtonStyled>

        <ButtonStyled circular type="transparent">
          <UiServersTeleportOverflowMenu :options="[...menuOptions]">
            <MoreVerticalIcon aria-hidden="true" />
            <template #kill>
              <SlashIcon class="h-5 w-5" />
              <span>Kill server</span>
            </template>
            <template #allServers>
              <ServerIcon class="h-5 w-5" />
              <span>All servers</span>
            </template>
            <template #details>
              <InfoIcon class="h-5 w-5" />
              <span>Details</span>
            </template>
            <template #copy-id>
              <ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
              <span>Copy ID</span>
            </template>
          </UiServersTeleportOverflowMenu>
        </ButtonStyled>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import {
  PlayIcon,
  UpdatedIcon,
  StopCircleIcon,
  SlashIcon,
  XIcon,
  CheckIcon,
  ServerIcon,
  InfoIcon,
  MoreVerticalIcon,
  ClipboardCopyIcon,
} from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { useRouter } from "vue-router";
import { useStorage } from "@vueuse/core";
import type { PowerAction as ServerPowerAction, ServerState } from "@modrinth/utils";

const flags = useFeatureFlags();

interface PowerAction {
  action: ServerPowerAction;
  nextState: ServerState;
}

const props = defineProps<{
  isOnline: boolean;
  isActioning: boolean;
  isInstalling: boolean;
  disabled: boolean;
  serverName?: string;
  serverData: object;
  uptimeSeconds: number;
}>();

const emit = defineEmits<{
  (e: "action", action: ServerPowerAction): void;
}>();

const router = useRouter();
const serverId = router.currentRoute.value.params.id;
const confirmActionModal = ref<InstanceType<typeof NewModal> | null>(null);
const detailsModal = ref<InstanceType<typeof NewModal> | null>(null);

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
  powerDontAskAgain: false,
});

const serverState = ref<ServerState>(props.isOnline ? "running" : "stopped");
const powerAction = ref<PowerAction | null>(null);
const dontAskAgain = ref(false);
const startingDelay = ref(false);

const canTakeAction = computed(
  () => !props.isActioning && !startingDelay.value && !isTransitionState.value,
);
const isRunning = computed(() => serverState.value === "running");
const isTransitionState = computed(() =>
  ["starting", "stopping", "restarting"].includes(serverState.value),
);
const isStoppingState = computed(() => serverState.value === "stopping");
const showStopButton = computed(() => isRunning.value || isStoppingState.value);

const primaryActionText = computed(() => {
  const states: Partial<Record<ServerState, string>> = {
    starting: "Starting...",
    restarting: "Restarting...",
    running: "Restart",
    stopping: "Stopping...",
    stopped: "Start",
  };
  return states[serverState.value];
});

const confirmActionText = computed(() => {
  if (!powerAction.value) return "";
  return powerAction.value.action.charAt(0).toUpperCase() + powerAction.value.action.slice(1);
});

const menuOptions = computed(() => [
  ...(props.isInstalling
    ? []
    : [
        {
          id: "kill",
          label: "Kill server",
          icon: SlashIcon,
          action: () => initiateAction("Kill"),
        },
      ]),
  {
    id: "allServers",
    label: "All servers",
    icon: ServerIcon,
    action: () => router.push("/servers/manage"),
  },
  {
    id: "details",
    label: "Details",
    icon: InfoIcon,
    action: () => detailsModal.value?.show(),
  },
  {
    id: "copy-id",
    label: "Copy ID",
    icon: ClipboardCopyIcon,
    action: () => copyId(),
    shown: flags.value.developerMode,
  },
]);

async function copyId() {
  await navigator.clipboard.writeText(serverId as string);
}

function initiateAction(action: ServerPowerAction) {
  if (!canTakeAction.value) return;

  const stateMap: Record<ServerPowerAction, ServerState> = {
    Start: "starting",
    Stop: "stopping",
    Restart: "restarting",
    Kill: "stopping",
  };

  if (action === "Start") {
    emit("action", action);
    serverState.value = stateMap[action];
    startingDelay.value = true;
    setTimeout(() => (startingDelay.value = false), 5000);
    return;
  }

  powerAction.value = { action, nextState: stateMap[action] };

  if (userPreferences.value.powerDontAskAgain) {
    executePowerAction();
  } else {
    confirmActionModal.value?.show();
  }
}

function handlePrimaryAction() {
  initiateAction(isRunning.value ? "Restart" : "Start");
}

function executePowerAction() {
  if (!powerAction.value) return;

  const { action, nextState } = powerAction.value;
  emit("action", action);
  serverState.value = nextState;

  if (dontAskAgain.value) {
    userPreferences.value.powerDontAskAgain = true;
  }

  if (action === "Start") {
    startingDelay.value = true;
    setTimeout(() => (startingDelay.value = false), 5000);
  }

  resetPowerAction();
}

function resetPowerAction() {
  confirmActionModal.value?.hide();
  powerAction.value = null;
  dontAskAgain.value = false;
}

function closeDetailsModal() {
  detailsModal.value?.hide();
}

watch(
  () => props.isOnline,
  (online) => (serverState.value = online ? "running" : "stopped"),
);

watch(
  () => router.currentRoute.value.fullPath,
  () => closeDetailsModal(),
);
</script>
