<template>
  <div class="flex flex-row items-center gap-2 rounded-lg">
    <ButtonStyled v-if="showStopButton" type="standard" color="red">
      <button :disabled="!canTakeAction" @click="stopServer">
        <div class="flex gap-1">
          <StopCircleIcon class="h-5 w-5" />
          <span>{{ stopButtonText }}</span>
        </div>
      </button>
    </ButtonStyled>

    <ButtonStyled type="standard" color="brand">
      <button :disabled="!canTakeAction" @click="handleAction">
        <div v-if="isStartingOrRestarting" class="grid place-content-center">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path
              fill-rule="evenodd"
              d="M15.312 11.424a5.5 5.5 0 0 1-9.201 2.466l-.312-.311h2.433a.75.75 0 0 0 0-1.5H3.989a.75.75 0 0 0-.75.75v4.242a.75.75 0 0 0 1.5 0v-2.43l.31.31a7 7 0 0 0 11.712-3.138.75.75 0 0 0-1.449-.39Zm1.23-3.723a.75.75 0 0 0 .219-.53V2.929a.75.75 0 0 0-1.5 0V5.36l-.31-.31A7 7 0 0 0 3.239 8.188a.75.75 0 1 0 1.448.389A5.5 5.5 0 0 1 13.89 6.11l.311.31h-2.432a.75.75 0 0 0 0 1.5h4.243a.75.75 0 0 0 .53-.219Z"
              clip-rule="evenodd"
            />
          </svg>
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
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { PlayIcon, UpdatedIcon, StopCircleIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const props = defineProps<{
  isOnline: boolean;
  isActioning: boolean;
}>();

const emit = defineEmits<{
  (e: "action", action: "start" | "restart" | "stop"): void;
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

const showStopButton = computed(
  () => currentState.value === ServerState.Running || currentState.value === ServerState.Stopping,
);
const showRestartIcon = computed(() => currentState.value === ServerState.Running);
const canTakeAction = computed(
  () =>
    currentState.value !== ServerState.Starting &&
    currentState.value !== ServerState.Stopping &&
    currentState.value !== ServerState.Restarting,
);
const isStartingOrRestarting = computed(
  () =>
    currentState.value === ServerState.Starting || currentState.value === ServerState.Restarting,
);

const actionButtonText = computed(() => {
  switch (currentState.value) {
    case ServerState.Stopped:
      return "Start";
    case ServerState.Starting:
      return "Starting...";
    case ServerState.Running:
      return "Restart";
    case ServerState.Restarting:
      return "Restarting...";
    case ServerState.Stopping:
      return "Restart";
    default:
      return "Unknown";
  }
});

const stopButtonText = computed(() => {
  return currentState.value === ServerState.Stopping ? "Stopping..." : "Stop";
});

const updateState = (newState: ServerStateType) => {
  currentState.value = newState;
};

const handleAction = () => {
  if (!canTakeAction.value) return;

  if (currentState.value === ServerState.Running) {
    updateState(ServerState.Restarting);
    emit("action", "restart");
  } else {
    updateState(ServerState.Starting);
    emit("action", "start");
  }
};

const stopServer = () => {
  if (!canTakeAction.value) return;

  updateState(ServerState.Stopping);
  emit("action", "stop");
};

watch(
  () => props.isOnline,
  (newValue) => {
    if (newValue) {
      updateState(ServerState.Running);
    } else {
      updateState(ServerState.Stopped);
    }
  },
);

// debounce to prevent flickering
watch(
  () => props.isActioning,
  (newValue) => {
    if (!newValue) {
      setTimeout(() => {
        if (
          currentState.value === ServerState.Starting ||
          currentState.value === ServerState.Restarting
        ) {
          updateState(ServerState.Running);
        } else if (currentState.value === ServerState.Stopping) {
          updateState(ServerState.Stopped);
        }
      }, 500);
    }
  },
);
</script>
