<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div class="flex h-full w-full flex-col">
      <div class="p-2 pt-0">
        <section v-if="error" class="universal-card">
          <h2>An error occurred whilst opening the task editor.</h2>
          <p>{{ error }}</p>
          <!-- TODO: Replace this with a better error visualization. -->
        </section>
        <section v-else class="universal-card">
          <h2>{{ isNew ? "New scheduled task" : "Editing scheduled task" }}</h2>
          <p class="mb-4">
            {{
              isNew
                ? "Create a new scheduled task to automatically perform actions at specific times."
                : "Modify this scheduled task's settings, timing, and actions."
            }}
          </p>

          <div class="mb-4 flex max-w-md flex-col gap-2">
            <label for="title">
              <span class="text-lg font-semibold text-contrast">
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

          <div class="mb-4 flex flex-col gap-2">
            <label><h3>Schedule Type</h3></label>
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

          <div
            v-if="scheduleType === 'daily'"
            class="card mb-4 flex max-w-md flex-row gap-16 rounded-lg !bg-bg p-4"
          >
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
              <TimePicker v-model="selectedTime" placeholder="Select time" />
            </div>
          </div>

          <div
            v-if="scheduleType === 'custom'"
            class="card mb-4 flex max-w-md flex-col gap-2 rounded-lg !bg-bg p-4"
          >
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
              placeholder="0 9 * * *"
            />
            <div v-if="!isValidCron" class="text-xs text-brand-red">
              Invalid cron format. Please use 5 space-separated values (e.g., minute hour day month
              day-of-week).
            </div>
            <div v-else class="text-xs">
              {{ humanReadableDescription }}
            </div>
          </div>

          <div class="mb-4 flex max-w-md flex-col gap-2">
            <label for="action_kind"><h3>Action</h3></label>
            <RadioButtons v-model="task.action_kind" :items="actionKinds" class="mb-2">
              <template #default="{ item }">
                <span>{{ item === "restart" ? "Restart server" : "Run game command" }}</span>
              </template>
            </RadioButtons>
          </div>

          <div v-if="task.action_kind === 'game-command'" class="mb-4 flex max-w-md flex-col gap-2">
            <label for="command"><h3>Command</h3></label>
            <input
              id="command"
              v-model="commandValue"
              type="text"
              maxlength="256"
              placeholder="Enter command to run..."
              class="input input-bordered"
            />
          </div>

          <div class="mb-4 flex max-w-md flex-col gap-2">
            <label for="warn_msg"
              ><h3>Warning Message</h3>
              <p>
                You can use <code>{}</code> to insert the time until the task is executed.
              </p></label
            >
            <input
              id="warn_msg"
              v-model="task.warn_msg"
              type="text"
              maxlength="128"
              :placeholder="`/tellraw @a \u0022Restarting in {}!\u0022`"
              class="input input-bordered"
            />
            <!-- TODO: Warning interval UI -->
          </div>

          <div class="mt-6 flex max-w-md gap-2">
            <ButtonStyled color="brand">
              <button :disabled="loading || !isValidCron || !task.title" @click="saveTask">
                <SaveIcon />
                Save
              </button>
            </ButtonStyled>
            <ButtonStyled>
              <button
                :disabled="loading"
                @click="() => router.push(`/servers/manage/${route.params.id}/options/scheduling`)"
              >
                <XIcon />
                Cancel
              </button>
            </ButtonStyled>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Schedule, ServerSchedule, ActionKind } from "@modrinth/utils";
import { ref, computed, onBeforeMount, watch } from "vue";
import RadioButtons from "@modrinth/ui/src/components/base/RadioButtons.vue";
import TimePicker from "@modrinth/ui/src/components/base/TimePicker.vue";
import { ButtonStyled } from "@modrinth/ui";
import { ClockIcon, CodeIcon, SaveIcon, XIcon, UnknownIcon } from "@modrinth/assets";
import { toString as cronToString } from "cronstrue";
import { ModrinthServer } from "~/composables/servers/modrinth-servers.ts";
import { useRoute, useRouter } from "#imports";

const props = defineProps<{ server: ModrinthServer }>();
const route = useRoute();
const router = useRouter();

const isNew = computed(() => route.params.taskId === "new");
const taskId = computed(() => (isNew.value ? null : Number(route.params.taskId)));
const loading = ref(false);
const error = ref<string | null>(null);
const task = ref<Partial<ServerSchedule | Schedule>>({});

const scheduleType = ref<"daily" | "custom">("daily");
const dayInterval = ref("1");
const selectedTime = ref({ hour: "9", minute: "0" });
const customCron = ref("0 9 * * *");
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
    return `${minute} ${hour} * * *`;
  } else {
    return `${minute} ${hour} */${days} * *`;
  }
});

const CRON_REGEX =
  /^\s*([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s+([0-9*/,-]+)\s*$/;

const isValidCron = computed(() => {
  const cronToTest = scheduleType.value === "custom" ? customCron.value.trim() : cronString.value;

  if (!CRON_REGEX.test(cronToTest)) {
    return false;
  }

  if (scheduleType.value === "custom") {
    try {
      cronToString(cronToTest);
      return true;
    } catch {
      return false;
    }
  }

  return true;
});

const humanReadableDescription = computed<string | undefined>(() => {
  if (!isValidCron.value && scheduleType.value === "custom") return undefined;
  if (scheduleType.value === "custom") {
    try {
      return cronToString(customCron.value.trim());
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

onBeforeMount(async () => {
  if (!isNew.value && taskId.value != null) {
    loading.value = true;
    error.value = null;
    try {
      if (!props.server.scheduling.tasks.length) {
        await props.server.scheduling.fetch();
      }
      const found = props.server.scheduling.tasks.find((t) => t.id === taskId.value);
      if (found) {
        task.value = { ...found };
        if (task.value.every && typeof task.value.every === "string") {
          const parts = task.value.every.split(/\s+/);
          if (parts.length === 5) {
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
              customCron.value = task.value.every;
            }
          } else {
            scheduleType.value = "custom";
            customCron.value = task.value.every;
          }
        }
      } else {
        error.value = "Task not found.";
      }
    } catch (e) {
      error.value = (e as Error).message || "Failed to load task.";
    } finally {
      loading.value = false;
    }
  } else {
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
    customCron.value = "0 9 * * *";
    if (isValidCron.value) {
      task.value.every = cronString.value;
    }
  }
});

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
    } else if (taskId.value != null) {
      await props.server.scheduling.editTask(taskId.value, task.value as Partial<Schedule>);
    }
    router.push(`/servers/manage/${route.params.id}/options/scheduling`);
  } catch (e) {
    error.value = (e as Error).message || "Failed to save task.";
  } finally {
    loading.value = false;
  }
}
</script>
