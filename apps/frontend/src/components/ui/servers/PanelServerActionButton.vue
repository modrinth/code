<template>
  <div class="flex flex-row items-center gap-2 rounded-lg">
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
          <UiServersLoadingIcon />
        </div>
        <div v-else class="contents">
          <component :is="showRestartIcon ? UpdatedIcon : PlayIcon" />
        </div>
        <span>
          {{ actionButtonText }}
        </span>
      </button>
    </ButtonStyled>

    <!-- kill option -->
    <ButtonStyled circular type="transparent">
      <UiServersTeleportOverflowMenu :options="[{ id: 'kill', action: () => killServer() }]">
        <MoreVerticalIcon aria-hidden="true" />
        <template #kill>
          <SlashIcon class="h-5 w-5" />
          <span>{{ killButtonText }}</span>
        </template>
      </UiServersTeleportOverflowMenu>
    </ButtonStyled>
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
} from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const props = defineProps<{
  isOnline: boolean;
  isActioning: boolean;
  disabled: boolean;
}>();

const emit = defineEmits<{
  (e: "action", action: "start" | "restart" | "stop" | "kill"): void;
}>();

const ServerState = {
  Stopped: "Stopped",
  Starting: "Starting",
  Running: "Running",
  Stopping: "Stopping",
  Restarting: "Restarting",
} as const;

type ServerStateType = (typeof ServerState)[keyof typeof ServerState];

const currentState = ref<ServerStateType>(
  props.isOnline ? ServerState.Running : ServerState.Stopped,
);

const isStartingDelay = ref(false);

const showStopButton = computed(() => currentState.value === ServerState.Running);
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
    default:
      return "Start";
  }
});

const stopButtonText = computed(() =>
  currentState.value === ServerState.Stopping ? "Stopping..." : "Stop",
);

const killButtonText = computed(() => "Kill");

const handleAction = () => {
  if (!canTakeAction.value) return;
  if (currentState.value === ServerState.Running) {
    currentState.value = ServerState.Restarting;
    emit("action", "restart");
  } else {
    currentState.value = ServerState.Starting;
    emit("action", "start");
    isStartingDelay.value = true;
    setTimeout(() => {
      isStartingDelay.value = false;
    }, 5000);
  }
};

const stopServer = () => {
  if (!canTakeAction.value) return;
  currentState.value = ServerState.Stopping;
  emit("action", "stop");
};

const killServer = () => {
  emit("action", "kill");
};

watch(
  () => props.isOnline,
  (newValue) => {
    if (newValue) {
      currentState.value = ServerState.Running;
    } else if (
      currentState.value !== ServerState.Starting &&
      currentState.value !== ServerState.Restarting
    ) {
      currentState.value = ServerState.Stopped;
    }
  },
);

watch(
  () => props.isActioning,
  (newValue) => {
    if (!newValue) {
      currentState.value = props.isOnline ? ServerState.Running : ServerState.Stopped;
    }
  },
);
</script>
