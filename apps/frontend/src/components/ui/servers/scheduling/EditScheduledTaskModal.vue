<template>
  <NewModal
    ref="modal"
    :header="isNew ? 'New scheduled task' : 'Editing scheduled task'"
    @hide="onModalHide"
  >
    <div class="modal-content flex flex-col gap-2">
      <div v-if="error" class="text-sm text-brand-red">
        {{ error }}
      </div>

      <div class="flex max-w-md flex-col gap-2">
        <label for="title">
          <span class="text-lg font-bold text-contrast">
            Title <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          id="title"
          v-model="task.title"
          type="text"
          maxlength="64"
          placeholder="Enter task title..."
          autocomplete="off"
          class="input input-bordered"
          required
        />
      </div>

      <div class="flex flex-col gap-2">
        <label><span class="text-lg font-bold text-contrast">Schedule Type</span></label>
        <RadioButtons v-model="scheduleType" :items="['daily', 'custom']" class="mb-2">
          <template #default="{ item }">
            <span class="flex items-center gap-2">
              <ClockIcon v-if="item === 'daily'" class="size-5" />
              <CodeIcon v-else class="size-5" />
              <span>{{
                item === "daily" ? "Every X day(s) at specific time" : "Custom cron expression"
              }}</span>
            </span>
          </template>
        </RadioButtons>
      </div>

      <div class="card flex h-[140px] items-center overflow-hidden rounded-lg !bg-bg">
        <Transition
          name="schedule-content"
          mode="out-in"
          enter-active-class="transition-all duration-300 ease-in-out"
          leave-active-class="transition-all duration-300 ease-in-out"
          enter-from-class="opacity-0"
          enter-to-class="opacity-100"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <div v-if="scheduleType === 'daily'" key="daily" class="flex w-full flex-col gap-2">
            <div class="flex w-full gap-8">
              <div class="flex flex-col gap-2">
                <label for="dayInterval" class="text-md font-medium">Every X day(s)</label>
                <div class="flex items-center gap-2">
                  <input
                    id="dayInterval"
                    v-model="dayInterval"
                    type="text"
                    inputmode="numeric"
                    maxlength="3"
                    class="input input-bordered w-20"
                    placeholder="1"
                    @input="onDayIntervalInput"
                  />
                  <span class="text-muted-foreground text-sm">day(s)</span>
                </div>
              </div>
              <div class="flex flex-col gap-2">
                <label class="text-md font-medium">At time</label>
                <TimePicker v-model="selectedTime" use-utc-values placeholder="Select time" />
              </div>
            </div>
            <div class="text-xs">
              {{ humanReadableDescription }}
            </div>
          </div>
          <div v-else key="custom" class="flex w-full flex-col gap-2">
            <label for="customCron" class="text-md font-medium"
              >Cron Expression
              <nuxt-link
                v-tooltip="
                  `Click to read more about what cron expressions are, and how to create them.`
                "
                class="align-middle text-link"
                target="_blank"
                to="https://www.geeksforgeeks.org/writing-cron-expressions-for-scheduling-tasks/"
                ><UnknownIcon class="size-4" />
              </nuxt-link>
            </label>
            <input
              id="customCron"
              v-model="customCron"
              type="text"
              class="input input-bordered font-mono"
              placeholder="0 0 9 * * *"
            />
            <div v-if="!isValidCron" class="text-xs text-brand-red">
              Invalid cron format. Please use 6 space-separated values (e.g.,
              <code>second minute hour day month day-of-week</code>).
            </div>
            <div v-else class="text-xs">
              {{ humanReadableDescription }}
            </div>
          </div>
        </Transition>
      </div>

      <div class="flex max-w-md flex-col gap-2">
        <label for="action_kind"><span class="text-lg font-bold text-contrast">Action</span></label>
        <RadioButtons v-model="task.action_kind" :items="actionKinds" class="mb-2">
          <template #default="{ item }">
            <span>{{ item === "restart" ? "Restart server" : "Run game command" }}</span>
          </template>
        </RadioButtons>
      </div>

      <Transition
        name="command-section"
        enter-active-class="transition-all duration-300 ease-in-out"
        leave-active-class="transition-all duration-300 ease-in-out"
        enter-from-class="opacity-0 max-h-0 overflow-hidden"
        enter-to-class="opacity-100 max-h-96 overflow-visible"
        leave-from-class="opacity-100 max-h-96 overflow-visible"
        leave-to-class="opacity-0 max-h-0 overflow-hidden"
      >
        <div v-if="task.action_kind === 'game-command'" class="flex max-w-lg flex-col gap-4">
          <label for="command" class="flex flex-col gap-2"
            ><span class="text-lg font-bold text-contrast">
              Command <span class="text-brand-red">*</span>
            </span>
            <span
              >The command that will be ran when the task is executed. If the command is invalid
              nothing will happen.</span
            ></label
          >
          <div class="max-w-md">
            <input
              id="command"
              v-model="commandValue"
              type="text"
              maxlength="256"
              placeholder="Enter command to run..."
              class="input input-bordered"
              required
            />
          </div>
        </div>
      </Transition>

      <div class="flex max-w-lg flex-col gap-4">
        <label for="warn_msg" class="flex flex-col gap-2"
          ><span class="text-lg font-bold text-contrast">
            Warning Command
            <span
              v-if="task.warn_intervals && task.warn_intervals.length > 0"
              class="text-brand-red"
              >*</span
            >
          </span>
          <span
            >You can use <code>{}</code> to insert the time until the task is executed. If the
            command is invalid nothing will happen.</span
          ></label
        >
        <div class="max-w-md">
          <input
            id="warn_msg"
            v-model="task.warn_msg"
            type="text"
            maxlength="128"
            :placeholder="`/tellraw @a \u0022Restarting in {}!\u0022`"
            class="input input-bordered max-w-md"
          />
        </div>
      </div>

      <div class="flex max-w-lg flex-col gap-4">
        <label for="warn_msg" class="flex flex-col gap-2"
          ><span class="text-lg font-bold text-contrast">Warning Intervals</span>
          <span
            >When, at the time before the scheduled task is executed, should the warning command be
            executed?</span
          ></label
        >
        <div class="flex flex-row flex-wrap gap-4">
          <Chips
            v-model="task.warn_intervals"
            :items="[5, 15, 30, 60, 120, 300, 600]"
            multi
            :never-empty="false"
            :format-label="formatWarningIntervalLabel"
          />
        </div>
      </div>

      <div class="mt-2 flex max-w-md gap-2">
        <ButtonStyled color="brand">
          <button :disabled="!canSave" @click="saveTask">
            <SaveIcon />
            Save
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button :disabled="loading" @click="closeModal">
            <XIcon />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import type { Schedule, ServerSchedule, ActionKind } from "@modrinth/utils";
import { ref, computed, watch } from "vue";
import { RadioButtons, TimePicker, Chips, ButtonStyled, NewModal } from "@modrinth/ui";
import { ClockIcon, CodeIcon, SaveIcon, XIcon, UnknownIcon } from "@modrinth/assets";
import { toString as cronToString } from "cronstrue";
import { ModrinthServer } from "~/composables/servers/modrinth-servers.ts";

const props = defineProps<{ server: ModrinthServer }>();

const modal = ref<InstanceType<typeof NewModal>>();
const loading = ref(false);
const error = ref<string | null>(null);
const task = ref<Partial<ServerSchedule | Schedule>>({});
const isNew = ref(true);
const originalTaskId = ref<number | null>(null);

const scheduleType = ref<"daily" | "custom">("daily");
const dayInterval = ref("1");
const selectedTime = ref({ hour: "9", minute: "0" });
const customCron = ref("0 0 9 * * *");
const actionKinds: ActionKind[] = ["restart", "game-command"];

const commandValue = computed({
  get() {
    return task.value.options && "command" in task.value.options ? task.value.options.command : "";
  },
  set(val: string) {
    if (task.value.options && "command" in task.value.options) {
      task.value.options.command = val;
    } else if (task.value.action_kind === "game-command") {
      task.value.options = { command: val };
    }
  },
});

const cronString = computed(() => {
  if (scheduleType.value === "custom") {
    return customCron.value.trim();
  }
  const minute = selectedTime.value.minute === "" ? "0" : selectedTime.value.minute;
  const hour = selectedTime.value.hour === "" ? "0" : selectedTime.value.hour;
  const days = dayInterval.value === "" || Number(dayInterval.value) < 1 ? "1" : dayInterval.value;
  if (days === "1") {
    return `0 ${minute} ${hour} * * *`;
  } else {
    return `0 ${minute} ${hour} */${days} * *`;
  }
});

const CRON_REGEX =
  /^\s*([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s*$/;

const isValidCron = computed(() => {
  const cronToTest = scheduleType.value === "custom" ? customCron.value.trim() : cronString.value;

  if (!CRON_REGEX.test(cronToTest)) {
    return false;
  }

  if (scheduleType.value === "custom") {
    try {
      const parts = cronToTest.split(/\s+/);
      if (parts.length === 6) {
        cronToString(parts.slice(1).join(" "));
      }
      return true;
    } catch {
      return false;
    }
  }

  return true;
});

const isValidGameCommand = computed(() => {
  if (task.value.action_kind === "game-command") {
    return commandValue.value && commandValue.value.trim().length > 0;
  }
  return true;
});

const isValidWarningCommand = computed(() => {
  if (task.value.warn_intervals && task.value.warn_intervals.length > 0) {
    return task.value.warn_msg && task.value.warn_msg.trim().length > 0;
  }
  return true;
});

const canSave = computed(() => {
  return (
    !loading.value &&
    task.value.title &&
    task.value.title.trim().length > 0 &&
    isValidCron.value &&
    isValidGameCommand.value &&
    isValidWarningCommand.value
  );
});

const humanReadableDescription = computed<string | undefined>(() => {
  if (!isValidCron.value && scheduleType.value === "custom") return undefined;
  if (scheduleType.value === "custom") {
    try {
      const parts = customCron.value.trim().split(/\s+/);
      if (parts.length === 6) {
        return cronToString(parts.slice(1).join(" "));
      }
      return "Invalid cron expression";
    } catch {
      return "Invalid cron expression";
    }
  }
  const minute = selectedTime.value.minute === "" ? "0" : selectedTime.value.minute;
  const hour = selectedTime.value.hour === "" ? "0" : selectedTime.value.hour;
  const daysNum = Number(dayInterval.value || "1");
  const timeStr = `${hour.padStart(2, "0")}:${minute.padStart(2, "0")}`;
  if (daysNum <= 1) {
    return `Every day at ${timeStr}`;
  } else {
    return `Every ${daysNum} days at ${timeStr}`;
  }
});

function formatWarningIntervalLabel(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  } else {
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m`;
  }
}

function onDayIntervalInput(event: Event) {
  const input = event.target as HTMLInputElement;
  let value = input.value.replace(/\D/g, "");

  if (value === "") {
    dayInterval.value = "";
    return;
  }

  if (value.length > 1 && value.startsWith("0")) {
    value = value.replace(/^0+/, "");
  }

  const num = parseInt(value);

  if (num < 1) {
    dayInterval.value = "1";
  } else if (num > 365) {
    dayInterval.value = "365";
  } else {
    dayInterval.value = String(num);
  }
}

function resetForm() {
  task.value = {
    title: "",
    action_kind: "restart",
    options: {},
    enabled: true,
    warn_msg: '/tellraw @a "Restarting in {}!"',
    warn_intervals: [],
  };
  scheduleType.value = "daily";
  dayInterval.value = "1";
  selectedTime.value = { hour: "9", minute: "0" };
  customCron.value = "0 0 9 * * *";
  isNew.value = true;
  originalTaskId.value = null;
  error.value = null;
  if (isValidCron.value) {
    task.value.every = cronString.value;
  }
}

function loadTaskData(taskData: ServerSchedule | Schedule) {
  task.value = { ...taskData };
  isNew.value = false;
  originalTaskId.value = "id" in taskData ? taskData.id : null;

  if (task.value.every && typeof task.value.every === "string") {
    const parts = task.value.every.split(/\s+/);
    if (parts.length === 6) {
      const second = parts[0];
      const minute = parts[1];
      const hour = parts[2];
      const dayOfMonth = parts[3];

      if (second === "0" && dayOfMonth.startsWith("*/")) {
        scheduleType.value = "daily";
        dayInterval.value = dayOfMonth.substring(2);
        selectedTime.value = { hour, minute };
      } else if (second === "0" && dayOfMonth === "*" && parts[4] === "*" && parts[5] === "*") {
        scheduleType.value = "daily";
        dayInterval.value = "1";
        selectedTime.value = { hour, minute };
      } else {
        scheduleType.value = "custom";
        customCron.value = task.value.every;
      }
    } else if (parts.length === 5) {
      const minute = parts[0];
      const hour = parts[1];
      const dayOfMonth = parts[2];

      if (dayOfMonth.startsWith("*/")) {
        scheduleType.value = "daily";
        dayInterval.value = dayOfMonth.substring(2);
        selectedTime.value = { hour, minute };
      } else if (dayOfMonth === "*" && parts[3] === "*" && parts[4] === "*") {
        scheduleType.value = "daily";
        dayInterval.value = "1";
        selectedTime.value = { hour, minute };
      } else {
        scheduleType.value = "custom";
        customCron.value = `0 ${task.value.every}`;
      }
    } else {
      scheduleType.value = "custom";
      customCron.value = task.value.every;
    }
  }
}

function show(taskData?: ServerSchedule | Schedule) {
  if (taskData) {
    loadTaskData(taskData);
  } else {
    resetForm();
  }
  modal.value?.show();
}

function closeModal() {
  modal.value?.hide();
}

function onModalHide() {
  // Reset form when modal is hidden
  resetForm();
}

async function saveTask() {
  loading.value = true;
  error.value = null;

  if (isValidCron.value) {
    task.value.every = cronString.value;
  } else {
    error.value = "Cannot save with invalid cron expression.";
    loading.value = false;
    return;
  }

  try {
    if (isNew.value) {
      const scheduleToCreate: Schedule = {
        title: task.value.title || "",
        every: task.value.every!,
        action_kind: task.value.action_kind!,
        options: task.value.options!,
        enabled: task.value.enabled !== undefined ? task.value.enabled : true,
        warn_msg: task.value.warn_msg !== undefined ? task.value.warn_msg : "",
        warn_intervals: task.value.warn_intervals || [],
      };
      await props.server.scheduling.createTask(scheduleToCreate);
      closeModal();
    } else if (originalTaskId.value != null) {
      await props.server.scheduling.editTask(originalTaskId.value, task.value);
      closeModal();
    }
  } catch (e) {
    error.value = (e as Error).message || "Failed to save task.";
  } finally {
    loading.value = false;
  }
}

watch([cronString, isValidCron], ([cron, valid]) => {
  if (valid) {
    task.value.every = cron;
  }
});

watch(
  () => task.value.action_kind,
  (kind) => {
    if (kind === "game-command") {
      if (
        !task.value.options ||
        typeof task.value.options !== "object" ||
        !("command" in task.value.options)
      ) {
        task.value.options = { command: "" };
      }
    } else {
      task.value.options = {};
    }
  },
  { immediate: true },
);

defineExpose({
  show,
  hide: closeModal,
});
</script>

<style scoped></style>
