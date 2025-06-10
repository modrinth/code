<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div class="flex h-full w-full flex-col">
      <section class="universal-card">
        <div class="mb-6 flex items-center justify-between">
          <div class="flex h-full flex-col justify-center">
            <h3 class="font-semibold leading-tight">Task Scheduling</h3>
            <p v-if="tasks.length < 1" class="mt-1 text-secondary">
              No scheduled tasks yet. Click the button to create your first task.
            </p>
            <p v-else class="mt-1 text-secondary">
              You can manage multiple tasks at once by selecting them below.
            </p>
          </div>
          <div>
            <NuxtLink
              :to="`/servers/manage/${route.params.id}/options/scheduling/new`"
              class="iconified-button brand-button"
            >
              <PlusIcon />
              Create Task
            </NuxtLink>
          </div>
        </div>

        <template v-if="tasks.length > 0">
          <div class="input-group">
            <ButtonStyled>
              <button :disabled="selectedTasks.length === 0" @click="handleBulkToggle">
                <ToggleRightIcon />
                Toggle Selected
              </button>
            </ButtonStyled>
            <ButtonStyled color="red">
              <button :disabled="selectedTasks.length === 0" @click="handleBulkDelete">
                <TrashIcon />
                Delete Selected
              </button>
            </ButtonStyled>
            <div class="push-right">
              <div class="labeled-control-row">
                Sort by
                <Multiselect
                  v-model="sortBy"
                  :searchable="false"
                  class="small-select"
                  :options="['Name', 'Type', 'Enabled', 'Schedule']"
                  :close-on-select="true"
                  :show-labels="false"
                  :allow-empty="false"
                  @update:model-value="updateSort"
                />
                <button
                  v-tooltip="descending ? 'Descending' : 'Ascending'"
                  class="square-button"
                  @click="updateDescending"
                >
                  <DescendingIcon v-if="descending" />
                  <AscendingIcon v-else />
                </button>
              </div>
            </div>
          </div>

          <div class="grid-table">
            <div class="grid-table__row grid-table__header">
              <div>
                <Checkbox
                  :model-value="selectedTasks.length === tasks.length && tasks.length > 0"
                  @update:model-value="handleSelectAll"
                />
              </div>
              <div>Type</div>
              <div>Task Details</div>
              <div>Schedule</div>
              <div>Warnings</div>
              <div>Enabled</div>
              <div>Actions</div>
            </div>

            <div
              v-for="(task, index) in sortedTasks"
              :key="`task-${index}`"
              class="grid-table__row"
            >
              <div>
                <Checkbox
                  :model-value="selectedTasks.includes(task)"
                  @update:model-value="(value) => handleTaskSelect(task, value)"
                />
              </div>
              <div>
                <RaisedBadge
                  :text="task.action_kind === 'restart' ? 'Restart' : 'Game Command'"
                  :icon="task.action_kind === 'restart' ? UpdatedIcon : CodeIcon"
                />
              </div>
              <div>
                <span class="mb-1 block font-medium text-primary">{{ task.title }}</span>
                <div
                  v-if="task.action_kind === 'game-command' && task.options?.command"
                  class="mt-1"
                >
                  <code
                    class="break-all rounded-sm bg-button-bg px-1 py-0.5 text-xs text-secondary"
                  >
                    {{ task.options.command }}
                  </code>
                </div>
              </div>
              <div>
                <span class="text-sm text-secondary">{{ getHumanReadableCron(task.every) }}</span>
              </div>
              <div>
                <div
                  v-if="task.warn_intervals && task.warn_intervals.length > 0"
                  class="flex flex-col gap-1"
                >
                  <span class="text-sm font-medium text-primary">
                    {{ task.warn_intervals.length }} warnings
                  </span>
                  <div class="font-mono text-xs text-secondary">
                    {{ formatWarningIntervals(task.warn_intervals) }}
                  </div>
                </div>
                <span v-else class="text-sm italic text-secondary">No warnings</span>
              </div>
              <div>
                <Toggle
                  :model-value="task.enabled"
                  @update:model-value="(value) => handleTaskToggle(task, value || false)"
                />
              </div>
              <div>
                <div class="flex gap-1">
                  <ButtonStyled icon-only circular>
                    <NuxtLink
                      v-tooltip="'Edit task'"
                      :to="`/servers/manage/${route.params.id}/options/scheduling/${task.id}`"
                    >
                      <EditIcon />
                    </NuxtLink>
                  </ButtonStyled>
                  <ButtonStyled icon-only circular color="red">
                    <button
                      v-tooltip="
                        task.title === 'Auto Restart'
                          ? 'You cant delete the automatic restart task.'
                          : 'Delete task'
                      "
                      :disabled="task.title === 'Auto Restart'"
                      @click="handleTaskDelete(task)"
                    >
                      <TrashIcon />
                    </button>
                  </ButtonStyled>
                </div>
              </div>
            </div>
          </div>
        </template>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { Multiselect } from "vue-multiselect";
import {
  PlusIcon,
  TrashIcon,
  ToggleRightIcon,
  EditIcon,
  UpdatedIcon,
  CodeIcon,
  SortAscendingIcon as AscendingIcon,
  SortDescendingIcon as DescendingIcon,
} from "@modrinth/assets";
import { Toggle, Checkbox, RaisedBadge, ButtonStyled } from "@modrinth/ui";
import cronstrue from "cronstrue";
import type { ServerSchedule } from "@modrinth/utils";
import { ModrinthServer } from "~/composables/servers/modrinth-servers.ts";

const props = defineProps<{
  server: ModrinthServer;
}>();

const route = useRoute();

onBeforeMount(async () => {
  await props.server.scheduling.fetch();
});

const selectedTasks = ref<ServerSchedule[]>([]);
const sortBy = ref("Name");
const descending = ref(false);

const tasks = computed(() => props.server.scheduling.tasks as ServerSchedule[]);

const sortedTasks = computed(() => {
  const sorted = [...tasks.value];

  switch (sortBy.value) {
    case "Name":
      sorted.sort((a, b) => a.title.localeCompare(b.title));
      break;
    case "Type":
      sorted.sort((a, b) => a.action_kind.localeCompare(b.action_kind));
      break;
    case "Enabled":
      sorted.sort((a, b) => {
        if (a.enabled === b.enabled) return 0;
        return a.enabled ? -1 : 1;
      });
      break;
    case "Schedule":
      sorted.sort((a, b) => a.every.localeCompare(b.every));
      break;
  }

  return descending.value ? sorted.reverse() : sorted;
});

function handleSelectAll(selected: boolean): void {
  selectedTasks.value = selected ? [...tasks.value] : [];
}

function handleTaskSelect(task: ServerSchedule, selected: boolean): void {
  if (selected) {
    selectedTasks.value.push(task);
  } else {
    selectedTasks.value = selectedTasks.value.filter((t) => t !== task);
  }
}

async function handleTaskToggle(task: ServerSchedule, enabled: boolean): Promise<void> {
  try {
    await props.server.scheduling.editTask(task.id, { enabled });
    console.log("Toggle task:", task.id, enabled);
  } catch (error) {
    console.error("Failed to toggle task:", error);
    task.enabled = !enabled;
  }
}

async function handleTaskDelete(task: ServerSchedule): Promise<void> {
  if (confirm(`Are you sure you want to delete "${task.title}"?`)) {
    try {
      await props.server.scheduling.deleteTask(task);
      selectedTasks.value = selectedTasks.value.filter((t) => t !== task);
      console.log("Delete task:", task.title);
    } catch (error) {
      console.error("Failed to delete task:", error);
    }
  }
}

async function handleBulkToggle(): Promise<void> {
  const enabledCount = selectedTasks.value.filter((t) => t.enabled).length;
  const shouldEnable = enabledCount < selectedTasks.value.length / 2;

  try {
    await Promise.all(
      selectedTasks.value.map((task) =>
        props.server.scheduling.editTask(task.id, { enabled: shouldEnable }),
      ),
    );
    console.log("Bulk toggle tasks:", selectedTasks.value.length, "to", shouldEnable);
  } catch (error) {
    console.error("Failed to bulk toggle tasks:", error);
  }
}

async function handleBulkDelete(): Promise<void> {
  if (confirm(`Are you sure you want to delete ${selectedTasks.value.length} selected tasks?`)) {
    try {
      await Promise.all(
        selectedTasks.value.map((task) => props.server.scheduling.deleteTask(task)),
      );
      selectedTasks.value = [];
      console.log("Bulk delete completed");
    } catch (error) {
      console.error("Failed to bulk delete tasks:", error);
    }
  }
}

function updateSort(): void {
  // Trigger reactivity for sortedTasks
}

function updateDescending(): void {
  descending.value = !descending.value;
}

function getHumanReadableCron(cronExpression: string): string {
  try {
    return cronstrue.toString(cronExpression);
  } catch {
    return cronExpression;
  }
}

function formatWarningIntervals(intervals: number[]): string {
  return intervals
    .sort((a, b) => b - a)
    .map((seconds) => {
      if (seconds >= 3600) return `${Math.floor(seconds / 3600)}h`;
      if (seconds >= 60) return `${Math.floor(seconds / 60)}m`;
      return `${seconds}s`;
    })
    .join(", ");
}
</script>

<style lang="scss" scoped>
.grid-table {
  display: grid;
  grid-template-columns:
    min-content minmax(min-content, 120px) minmax(min-content, 2fr)
    minmax(min-content, 1fr) minmax(min-content, 120px) min-content min-content;
  border-radius: var(--size-rounded-sm);
  overflow: hidden;
  margin-top: var(--spacing-card-md);
  outline: 1px solid transparent;

  .grid-table__row {
    display: contents;

    > div {
      display: flex;
      flex-direction: column;
      justify-content: center;
      padding: var(--spacing-card-sm);

      &:first-child {
        padding-left: var(--spacing-card-bg);
      }

      &:last-child {
        padding-right: var(--spacing-card-bg);
      }
    }

    &:nth-child(2n + 1) > div {
      background-color: var(--color-table-alternate-row);
    }

    &.grid-table__header > div {
      background-color: var(--color-bg);
      font-weight: bold;
      color: var(--color-text-dark);
      padding-top: var(--spacing-card-bg);
      padding-bottom: var(--spacing-card-bg);
    }
  }

  @media screen and (max-width: 900px) {
    display: flex;
    flex-direction: column;

    .grid-table__row {
      display: grid;
      grid-template:
        "checkbox type title actions"
        "checkbox schedule warnings actions";
      grid-template-columns: min-content min-content 1fr min-content;

      :nth-child(1) {
        grid-area: checkbox;
      }
      :nth-child(2) {
        grid-area: type;
      }
      :nth-child(3) {
        grid-area: title;
      }
      :nth-child(4) {
        grid-area: schedule;
        padding-top: 0;
      }
      :nth-child(5) {
        grid-area: warnings;
        padding-top: 0;
      }
      :nth-child(6) {
        grid-area: actions;
        align-items: flex-start;
      }
      :nth-child(7) {
        grid-area: actions;
        align-items: flex-end;
      }
    }

    .grid-table__header {
      grid-template: "checkbox actions";
      grid-template-columns: min-content minmax(min-content, 1fr);

      :nth-child(2),
      :nth-child(3),
      :nth-child(4),
      :nth-child(5),
      :nth-child(6) {
        display: none;
      }
    }
  }

  @media screen and (max-width: 600px) {
    .grid-table__row :nth-child(3) code {
      font-size: 11px;
    }

    .grid-table__row :nth-child(5) .font-mono {
      font-size: 11px;
    }
  }
}

.labeled-control-row {
  flex: 1;
  display: flex;
  flex-direction: row;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-card-md);
  white-space: nowrap;
}

.small-select {
  width: -moz-fit-content;
  width: fit-content;
}

.push-right {
  margin-left: auto;
}
</style>
