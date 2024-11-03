<template>
  <div class="contents">
    <NewModal ref="confirmActionModal" header="Confirming power action" @close="closePowerModal">
      <div class="flex flex-col gap-4 md:w-[400px]">
        <p class="m-0">Are you sure you want to {{ currentPendingAction }} the server?</p>

        <UiCheckbox
          v-model="powerDontAskAgainCheckbox"
          label="Don't ask me again"
          class="text-sm"
          :disabled="!currentPendingAction"
        />
        <div class="flex flex-row gap-4">
          <ButtonStyled type="standard" color="brand" @click="confirmAction">
            <button>
              <CheckIcon class="h-5 w-5" />
              {{ currentPendingActionFriendly }} server
            </button>
          </ButtonStyled>
          <ButtonStyled @click="closePowerModal">
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
      :header="`All of ${props.serverName ? props.serverName : 'Server'} info`"
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
      <div v-else class="contents">
        <ButtonStyled v-if="showStopButton" type="transparent">
          <button :disabled="!canTakeAction || disabled || isStopping" @click="stopServer">
            <div class="flex gap-1">
              <StopCircleIcon class="h-5 w-5" />
              <span>{{ stopButtonText }}</span>
            </div>
          </button>
        </ButtonStyled>
        <ButtonStyled type="standard" color="brand">
          <button :disabled="!canTakeAction || disabled || isStopping" @click="handleAction">
            <div v-if="isStartingOrRestarting" class="grid place-content-center">
              <UiServersIconsLoadingIcon />
            </div>
            <div v-else class="contents">
              <component :is="showRestartIcon ? UpdatedIcon : PlayIcon" />
            </div>
            <span>
              {{ actionButtonText }}
            </span>
          </button>
        </ButtonStyled>
      </div>

      <!-- Dropdown options -->
      <ButtonStyled circular type="transparent">
        <UiServersTeleportOverflowMenu
          :options="[
            ...(props.isInstalling ? [] : [{ id: 'kill', action: () => killServer() }]),
            { id: 'allServers', action: () => router.push('/servers/manage') },
            { id: 'details', action: () => showDetailsModal() },
          ]"
        >
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
        </UiServersTeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import {
  PlayIcon,
  UpdatedIcon,
  StopCircleIcon,
  SlashIcon,
  MoreVerticalIcon,
  XIcon,
  CheckIcon,
  ServerIcon,
  InfoIcon,
} from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { useRouter } from "vue-router";
import { useStorage } from "@vueuse/core";

const props = defineProps<{
  isOnline: boolean;
  isActioning: boolean;
  isInstalling: boolean;
  disabled: boolean;
  serverName?: string;
  serverData: object;
  uptimeSeconds: number;
}>();

const router = useRouter();
const serverId = router.currentRoute.value.params.id;

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
  powerDontAskAgain: false,
});

const emit = defineEmits<{
  (e: "action", action: "start" | "restart" | "stop" | "kill"): void;
}>();

const confirmActionModal = ref<InstanceType<typeof NewModal> | null>(null);
const detailsModal = ref<InstanceType<typeof NewModal> | null>(null);

const ServerState = {
  Stopped: "Stopped",
  Starting: "Starting",
  Running: "Running",
  Stopping: "Stopping",
  Restarting: "Restarting",
} as const;

type ServerStateType = (typeof ServerState)[keyof typeof ServerState];

const currentPendingAction = ref<string | null>(null);
const currentPendingState = ref<ServerStateType | null>(null);
const powerDontAskAgainCheckbox = ref(false);

const currentState = ref<ServerStateType>(
  props.isOnline ? ServerState.Running : ServerState.Stopped,
);

const isStartingDelay = ref(false);
const showStopButton = computed(
  () => currentState.value === ServerState.Running || currentState.value === ServerState.Stopping,
);
const showRestartIcon = computed(() => currentState.value === ServerState.Running);
const canTakeAction = computed(
  () =>
    !props.isActioning &&
    !isStartingDelay.value &&
    currentState.value !== ServerState.Starting &&
    currentState.value !== ServerState.Stopping,
);

const isStartingOrRestarting = computed(
  () =>
    currentState.value === ServerState.Starting || currentState.value === ServerState.Restarting,
);

const isStopping = computed(() => currentState.value === ServerState.Stopping);

const actionButtonText = computed(() => {
  switch (currentState.value) {
    case ServerState.Starting:
      return "Starting...";
    case ServerState.Restarting:
      return "Restarting...";
    case ServerState.Running:
      return "Restart";
    case ServerState.Stopping:
      return "Stopping...";
    default:
      return "Start";
  }
});

const currentPendingActionFriendly = computed(() => {
  switch (currentPendingAction.value) {
    case "start":
      return "Start";
    case "restart":
      return "Restart";
    case "stop":
      return "Stop";
    case "kill":
      return "Kill";
    default:
      return null;
  }
});

const stopButtonText = computed(() =>
  currentState.value === ServerState.Stopping ? "Stopping..." : "Stop",
);

const createPendingAction = () => {
  if (!canTakeAction.value) return;
  if (currentState.value === ServerState.Running) {
    currentPendingAction.value = "restart";
    currentPendingState.value = ServerState.Restarting;
    showPowerModal();
  } else {
    runAction("start", ServerState.Starting);
  }
};

const handleAction = () => {
  createPendingAction();
};

const showPowerModal = () => {
  if (userPreferences.value.powerDontAskAgain) {
    runAction(
      currentPendingAction.value as "start" | "restart" | "stop" | "kill",
      currentPendingState.value!,
    );
  } else {
    confirmActionModal.value?.show();
  }
};

const confirmAction = () => {
  if (powerDontAskAgainCheckbox.value) {
    userPreferences.value.powerDontAskAgain = true;
  }
  runAction(
    currentPendingAction.value as "start" | "restart" | "stop" | "kill",
    currentPendingState.value!,
  );
  closePowerModal();
};

const runAction = (action: "start" | "restart" | "stop" | "kill", serverState: ServerStateType) => {
  emit("action", action);
  currentState.value = serverState;

  if (action === "start") {
    isStartingDelay.value = true;
    setTimeout(() => {
      isStartingDelay.value = false;
    }, 5000);
  }
};

const stopServer = () => {
  if (!canTakeAction.value) return;
  currentPendingAction.value = "stop";
  currentPendingState.value = ServerState.Stopping;
  showPowerModal();
};

const killServer = () => {
  currentPendingAction.value = "kill";
  currentPendingState.value = ServerState.Stopping;
  showPowerModal();
};

const closePowerModal = () => {
  confirmActionModal.value?.hide();
  currentPendingAction.value = null;
  powerDontAskAgainCheckbox.value = false;
};

const closeDetailsModal = () => {
  detailsModal.value?.hide();
};

const showDetailsModal = () => {
  detailsModal.value?.show();
};

watch(
  () => props.isOnline,
  (newValue) => {
    if (newValue) {
      currentState.value = ServerState.Running;
    } else {
      currentState.value = ServerState.Stopped;
    }
  },
);

watch(
  () => router.currentRoute.value.fullPath,
  () => {
    closeDetailsModal();
  },
);
</script>
