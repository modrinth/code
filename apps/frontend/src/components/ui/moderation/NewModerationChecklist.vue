<template>
  <KeybindsModal ref="keybindsModal" />
  <div
    tabindex="0"
    class="moderation-checklist flex w-[600px] max-w-full flex-col rounded-2xl border-[1px] border-solid border-orange bg-bg-raised p-4 transition-all delay-200 duration-200 ease-in-out"
    :class="collapsed ? '!w-fit' : ''"
  >
    <div class="flex grow-0 items-center gap-2">
      <h1 class="m-0 mr-auto flex items-center gap-2 text-2xl font-extrabold text-contrast">
        <ScaleIcon class="text-orange" /> Moderation
      </h1>
      <ButtonStyled circular>
        <button v-tooltip="`Keyboard shortcuts`" @click="keybindsModal?.show($event)">
          <KeyboardIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled circular>
        <a v-tooltip="`Stage guidance`" target="_blank" :href="currentStageObj.guidance_url">
          <FileTextIcon />
        </a>
      </ButtonStyled>
      <ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
        <button v-tooltip="`Reset progress`" @click="resetProgress">
          <BrushCleaningIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
        <button v-tooltip="`Exit moderation`" @click="emit('exit')">
          <XIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled circular>
        <button v-tooltip="collapsed ? `Expand` : `Collapse`" @click="emit('toggleCollapsed')">
          <DropdownIcon class="transition-transform" :class="{ 'rotate-180': collapsed }" />
        </button>
      </ButtonStyled>
    </div>

    <Collapsible base-class="grow" class="flex grow flex-col" :collapsed="collapsed">
      <div class="my-4 h-[1px] w-full bg-divider" />
      <div class="flex-1">
        <div v-if="done">
          <p>
            You are done moderating this project!
            <template v-if="futureProjectCount > 0">
              There are
              {{ futureProjectCount }} left.
            </template>
          </p>
        </div>
        <div v-else-if="generatedMessage">
          <div>
            <MarkdownEditor
              v-model="message"
              :max-height="400"
              placeholder="No message generated."
              :disabled="false"
              :heading-buttons="false"
            />
          </div>
        </div>
        <div v-else-if="isModpackPermissionsStage">
          <ModpackPermissionsFlow
            v-model="modpackJudgements"
            :project-id="project.id"
            @complete="handleModpackPermissionsComplete"
          />
        </div>
        <div v-else>
          <h2 class="m-0 mb-2 text-lg font-extrabold">
            {{ currentStageObj.title }}
          </h2>

          <div v-if="currentStageObj.text" class="mb-4">
            <div v-if="stageTextExpanded" class="markdown-body" v-html="stageTextExpanded"></div>
            <div v-else class="markdown-body">Loading stage content...</div>
          </div>

          <!-- Action components grouped by type -->
          <div class="space-y-4">
            <!-- Button actions group -->
            <div v-if="buttonActions.length > 0" class="button-actions-group">
              <div class="flex flex-wrap gap-2">
                <template v-for="action in buttonActions" :key="getActionKey(action)">
                  <ButtonStyled
                    :color="isActionSelected(action) ? 'brand' : 'standard'"
                    @click="toggleAction(action)"
                  >
                    <button>
                      {{ action.label }}
                    </button>
                  </ButtonStyled>
                </template>
              </div>
            </div>

            <!-- Toggle actions group -->
            <div v-if="toggleActions.length > 0" class="toggle-actions-group space-y-3">
              <template v-for="action in toggleActions" :key="getActionKey(action)">
                <Checkbox
                  :model-value="actionStates[getActionId(action)]?.selected ?? false"
                  :label="action.label"
                  :description="action.description"
                  :disabled="false"
                  @update:model-value="toggleAction(action)"
                />
              </template>
            </div>

            <!-- Dropdown actions group -->
            <div v-if="dropdownActions.length > 0" class="dropdown-actions-group space-y-3">
              <template v-for="action in dropdownActions" :key="getActionKey(action)">
                <div class="inputs universal-labels">
                  <div>
                    <label :for="`dropdown-${getActionId(action)}`">
                      <span class="label__title">{{ action.label }}</span>
                    </label>
                    <DropdownSelect
                      :max-visible-options="3"
                      render-up
                      :name="`dropdown-${getActionId(action)}`"
                      :options="getVisibleDropdownOptions(action)"
                      :model-value="getDropdownValue(action)"
                      :placeholder="'Select an option'"
                      :disabled="false"
                      :display-name="(opt: any) => opt?.label || 'Unknown option'"
                      @update:model-value="
                        (selected: any) => selectDropdownOption(action, selected)
                      "
                    />
                  </div>
                </div>
              </template>
            </div>

            <!-- Multi-select chips actions group -->
            <div v-if="multiSelectActions.length > 0" class="multi-select-actions-group space-y-3">
              <template v-for="action in multiSelectActions" :key="getActionKey(action)">
                <div>
                  <div class="mb-2 font-semibold">{{ action.label }}</div>
                  <div class="flex flex-wrap gap-2">
                    <ButtonStyled
                      v-for="(option, optIndex) in getVisibleMultiSelectOptions(action)"
                      :key="`${getActionId(action)}-chip-${optIndex}`"
                      :color="isChipSelected(action, optIndex) ? 'brand' : 'standard'"
                      @click="toggleChip(action, optIndex)"
                    >
                      <button>
                        {{ option.label }}
                      </button>
                    </ButtonStyled>
                  </div>
                </div>
              </template>
            </div>
          </div>

          <div v-if="isAnyVisibleInputs" class="my-4 h-[1px] w-full bg-divider" />

          <!-- Additional text inputs -->
          <div class="space-y-4">
            <template v-for="action in visibleActions" :key="`inputs-${getActionKey(action)}`">
              <div
                v-if="action.relevantExtraInput && isActionSelected(action)"
                class="inputs universal-labels"
              >
                <div
                  v-for="(input, inputIndex) in getVisibleInputs(action, actionStates)"
                  :key="`input-${getActionId(action)}-${inputIndex}`"
                  class="mt-2"
                >
                  <template v-if="input.large">
                    <label :for="`input-${getActionId(action)}-${inputIndex}`">
                      <span class="label__title">
                        {{ input.label }}
                        <span v-if="input.required" class="required">*</span>
                      </span>
                    </label>
                    <MarkdownEditor
                      :id="`input-${getActionId(action)}-${inputIndex}`"
                      v-model="textInputValues[`${getActionId(action)}-${inputIndex}`]"
                      :placeholder="input.placeholder"
                      :max-height="300"
                      :disabled="false"
                      :heading-buttons="false"
                      @input="persistState"
                    />
                  </template>
                  <template v-else>
                    <label :for="`input-${getActionId(action)}-${inputIndex}`">
                      <span class="label__title">
                        {{ input.label }}
                        <span v-if="input.required" class="required">*</span>
                      </span>
                    </label>
                    <input
                      :id="`input-${getActionId(action)}-${inputIndex}`"
                      v-model="textInputValues[`${getActionId(action)}-${inputIndex}`]"
                      type="text"
                      :placeholder="input.placeholder"
                      autocomplete="off"
                      @input="persistState"
                    />
                  </template>
                </div>
              </div>
            </template>
          </div>
        </div>
      </div>

      <!-- Stage control buttons -->
      <div class="mt-auto">
        <div
          class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-divider pt-4"
        >
          <div class="flex items-center gap-2">
            <ButtonStyled v-if="!done && !generatedMessage && futureProjectCount > 0">
              <button @click="goToNextProject">
                <XIcon aria-hidden="true" />
                Skip
              </button>
            </ButtonStyled>
          </div>

          <div class="flex items-center gap-2">
            <div v-if="done">
              <ButtonStyled v-if="futureProjectCount > 0" color="brand">
                <button @click="goToNextProject">
                  <RightArrowIcon aria-hidden="true" />
                  Next Project
                </button>
              </ButtonStyled>
              <ButtonStyled v-else color="brand">
                <button @click="exitModeration">
                  <CheckIcon aria-hidden="true" />
                  Done
                </button>
              </ButtonStyled>
            </div>

            <div v-else-if="generatedMessage" class="flex items-center gap-2">
              <OverflowMenu :options="stageOptions" class="bg-transparent p-0">
                <ButtonStyled circular>
                  <button v-tooltip="`Stages`">
                    <ListBulletedIcon />
                  </button>
                </ButtonStyled>

                <template
                  v-for="opt in stageOptions.filter(
                    (opt) => 'id' in opt && 'text' in opt && 'icon' in opt,
                  )"
                  #[opt.id]
                  :key="opt.id"
                >
                  <component :is="opt.icon" v-if="opt.icon" class="mr-2" />
                  {{ opt.text }}
                </template>
              </OverflowMenu>
              <ButtonStyled>
                <button @click="goBackToStages">
                  <LeftArrowIcon aria-hidden="true" />
                  Edit
                </button>
              </ButtonStyled>
              <ButtonStyled color="red">
                <button @click="sendMessage('rejected')">
                  <XIcon aria-hidden="true" />
                  Reject
                </button>
              </ButtonStyled>
              <ButtonStyled color="orange">
                <button @click="sendMessage('withheld')">
                  <EyeOffIcon aria-hidden="true" />
                  Withhold
                </button>
              </ButtonStyled>
              <ButtonStyled color="green">
                <button @click="sendMessage('approved')">
                  <CheckIcon aria-hidden="true" />
                  Approve
                </button>
              </ButtonStyled>
            </div>

            <div v-else class="flex items-center gap-2">
              <OverflowMenu
                v-if="!generatedMessage"
                :options="stageOptions"
                class="bg-transparent p-0"
              >
                <ButtonStyled circular>
                  <button v-tooltip="`Stages`">
                    <ListBulletedIcon />
                  </button>
                </ButtonStyled>

                <template
                  v-for="opt in stageOptions.filter(
                    (opt) => 'id' in opt && 'text' in opt && 'icon' in opt,
                  )"
                  #[opt.id]
                  :key="opt.id"
                >
                  <component :is="opt.icon" v-if="opt.icon" class="mr-2" />
                  {{ opt.text }}
                </template>
              </OverflowMenu>
              <ButtonStyled>
                <button :disabled="!hasValidPreviousStage" @click="previousStage">
                  <LeftArrowIcon aria-hidden="true" /> Previous
                </button>
              </ButtonStyled>
              <ButtonStyled v-if="!isLastVisibleStage" color="brand">
                <button @click="nextStage"><RightArrowIcon aria-hidden="true" /> Next</button>
              </ButtonStyled>
              <ButtonStyled v-else color="brand" :disabled="loadingMessage">
                <button @click="generateMessage">
                  <CheckIcon aria-hidden="true" />
                  {{ loadingMessage ? "Generating..." : "Generate Message" }}
                </button>
              </ButtonStyled>
            </div>
          </div>
        </div>
      </div>
    </Collapsible>
  </div>
</template>

<script lang="ts" setup>
import {
  LeftArrowIcon,
  RightArrowIcon,
  DropdownIcon,
  XIcon,
  ScaleIcon,
  ListBulletedIcon,
  FileTextIcon,
  BrushCleaningIcon,
  CheckIcon,
  KeyboardIcon,
  EyeOffIcon,
} from "@modrinth/assets";
import {
  checklist,
  getActionIdForStage,
  initializeActionState,
  getActionMessage,
  findMatchingVariant,
  processMessage,
  getVisibleInputs,
  serializeActionStates,
  deserializeActionStates,
  kebabToTitleCase,
  flattenProjectVariables,
  expandVariables,
  handleKeybind,
  keybinds,
} from "@modrinth/moderation";
import {
  ButtonStyled,
  Collapsible,
  OverflowMenu,
  type OverflowMenuOption,
  Checkbox,
  DropdownSelect,
  MarkdownEditor,
} from "@modrinth/ui";
import { type Project, renderHighlightedString, type ModerationJudgements } from "@modrinth/utils";
import { computedAsync, useLocalStorage } from "@vueuse/core";
import type {
  Action,
  MultiSelectChipsAction,
  DropdownAction,
  ButtonAction,
  ToggleAction,
  ConditionalButtonAction,
  Stage,
} from "@modrinth/moderation";
import ModpackPermissionsFlow from "./ModpackPermissionsFlow.vue";
import KeybindsModal from "./ChecklistKeybindsModal.vue";
import { finalPermissionMessages } from "@modrinth/moderation/data/modpack-permissions-stage";
import prettier from "prettier";

const keybindsModal = ref<InstanceType<typeof KeybindsModal>>();

const props = withDefaults(
  defineProps<{
    project: Project;
    futureProjectIds?: string[];
    collapsed: boolean;
  }>(),
  {
    futureProjectIds: () => [] as string[],
  },
);

const variables = computed(() => {
  return flattenProjectVariables(props.project);
});

const futureProjectCount = computed(() => {
  const ids = JSON.parse(localStorage.getItem("moderation-future-projects") || "[]");
  return ids.length;
});

const modpackPermissionsComplete = ref(false);
const modpackJudgements = ref<ModerationJudgements>({});
const isModpackPermissionsStage = computed(() => {
  return currentStageObj.value.id === "modpack-permissions";
});

const message = ref("");
const generatedMessage = ref(false);
const loadingMessage = ref(false);
const done = ref(false);

function handleModpackPermissionsComplete() {
  modpackPermissionsComplete.value = true;
  nextStage();
}

const emit = defineEmits<{
  exit: [];
  toggleCollapsed: [];
}>();

function resetProgress() {
  currentStage.value = findFirstValidStage();
  actionStates.value = {};
  textInputValues.value = {};

  done.value = false;
  generatedMessage.value = false;
  message.value = "";
  loadingMessage.value = false;

  localStorage.removeItem(`modpack-permissions-${props.project.id}`);
  localStorage.removeItem(`modpack-permissions-index-${props.project.id}`);
  modpackPermissionsComplete.value = false;
  modpackJudgements.value = {};

  initializeAllStages();
}

function findFirstValidStage(): number {
  for (let i = 0; i < checklist.length; i++) {
    if (shouldShowStageIndex(i)) {
      return i;
    }
  }
  return 0;
}

const currentStageObj = computed(() => checklist[currentStage.value]);
const currentStage = useLocalStorage(`moderation-stage-${props.project.slug}`, () =>
  findFirstValidStage(),
);

const stageTextExpanded = computedAsync(async () => {
  const stageIndex = currentStage.value;
  const stage = checklist[stageIndex];
  if (stage.text) {
    return renderHighlightedString(
      expandVariables(await stage.text(), props.project, variables.value),
    );
  }
  return null;
}, null);

interface ActionState {
  selected: boolean;
  value?: any;
}

const persistedActionStates = useLocalStorage(
  `moderation-actions-${props.project.slug}`,
  {},
  {
    serializer: {
      read: (v: any) => (v ? deserializeActionStates(v) : {}),
      write: (v: any) => serializeActionStates(v),
    },
  },
);

const router = useRouter();

const persistedTextInputs = useLocalStorage(
  `moderation-inputs-${props.project.slug}`,
  {} as Record<string, string>,
);

const actionStates = ref<Record<string, ActionState>>(persistedActionStates.value);
const textInputValues = ref<Record<string, string>>(persistedTextInputs.value);

const persistState = () => {
  persistedActionStates.value = actionStates.value;
  persistedTextInputs.value = textInputValues.value;
};

watch(actionStates, persistState, { deep: true });
watch(textInputValues, persistState, { deep: true });

interface MessagePart {
  weight: number;
  content: string;
  actionId: string;
  stageIndex: number;
}

function handleKeybinds(event: KeyboardEvent) {
  const focusedActionIndex = ref<number | null>(null);

  handleKeybind(
    event,
    {
      project: props.project,
      state: {
        currentStage: currentStage.value,
        totalStages: checklist.length,
        currentStageId: currentStageObj.value.id,
        currentStageTitle: currentStageObj.value.title,

        isCollapsed: props.collapsed,
        isDone: done.value,
        hasGeneratedMessage: generatedMessage.value,
        isLoadingMessage: loadingMessage.value,
        isModpackPermissionsStage: isModpackPermissionsStage.value,

        futureProjectCount: futureProjectCount.value,
        visibleActionsCount: visibleActions.value.length,

        focusedActionIndex: focusedActionIndex.value,
        focusedActionType:
          focusedActionIndex.value !== null
            ? (visibleActions.value[focusedActionIndex.value]?.type as any)
            : null,
      },
      actions: {
        tryGoNext: nextStage,
        tryGoBack: previousStage,
        tryGenerateMessage: generateMessage,
        trySkipProject: goToNextProject,

        tryToggleCollapse: () => emit("toggleCollapsed"),
        tryResetProgress: resetProgress,
        tryExitModeration: () => emit("exit"),

        tryApprove: () => sendMessage("approved"),
        tryReject: () => sendMessage("rejected"),
        tryWithhold: () => sendMessage("withheld"),
        tryEditMessage: goBackToStages,

        tryToggleAction: (actionIndex: number) => {
          const action = visibleActions.value[actionIndex];
          if (action) {
            toggleAction(action);
          }
        },
        trySelectDropdownOption: (actionIndex: number, optionIndex: number) => {
          const action = visibleActions.value[actionIndex] as DropdownAction;
          if (action && action.type === "dropdown") {
            const visibleOptions = getVisibleDropdownOptions(action);
            if (optionIndex < visibleOptions.length) {
              selectDropdownOption(action, visibleOptions[optionIndex]);
            }
          }
        },
        tryToggleChip: (actionIndex: number, chipIndex: number) => {
          const action = visibleActions.value[actionIndex] as MultiSelectChipsAction;
          if (action && action.type === "multi-select-chips") {
            const visibleOptions = getVisibleMultiSelectOptions(action);
            if (chipIndex < visibleOptions.length) {
              toggleChip(action, chipIndex);
            }
          }
        },

        tryFocusNextAction: () => {
          if (visibleActions.value.length === 0) return;
          if (focusedActionIndex.value === null) {
            focusedActionIndex.value = 0;
          } else {
            focusedActionIndex.value = (focusedActionIndex.value + 1) % visibleActions.value.length;
          }
        },
        tryFocusPreviousAction: () => {
          if (visibleActions.value.length === 0) return;
          if (focusedActionIndex.value === null) {
            focusedActionIndex.value = visibleActions.value.length - 1;
          } else {
            focusedActionIndex.value =
              focusedActionIndex.value === 0
                ? visibleActions.value.length - 1
                : focusedActionIndex.value - 1;
          }
        },
        tryActivateFocusedAction: () => {
          if (focusedActionIndex.value === null) return;
          const action = visibleActions.value[focusedActionIndex.value];
          if (!action) return;

          if (
            action.type === "button" ||
            action.type === "conditional-button" ||
            action.type === "toggle"
          ) {
            toggleAction(action);
          }
        },
      },
    },
    keybinds,
  );
}

onMounted(() => {
  window.addEventListener("keydown", handleKeybinds);
  initializeAllStages();
});

function initializeAllStages() {
  checklist.forEach((stage, stageIndex) => {
    initializeStageActions(stage, stageIndex);
  });
}

function initializeCurrentStage() {
  initializeStageActions(currentStageObj.value, currentStage.value);
}

watch(
  currentStage,
  (newIndex) => {
    const stage = checklist[newIndex];
    if (stage?.navigate) {
      router.push(`/${props.project.project_type}/${props.project.slug}${stage.navigate}`);
    }

    initializeCurrentStage();
  },
  { immediate: true },
);

function initializeStageActions(stage: Stage, stageIndex: number) {
  stage.actions.forEach((action, index) => {
    const actionId = getActionIdForStage(action, stageIndex, index);
    if (!actionStates.value[actionId]) {
      actionStates.value[actionId] = initializeActionState(action);
    }
  });

  stage.actions.forEach((action) => {
    if (action.enablesActions) {
      action.enablesActions.forEach((enabledAction, index) => {
        const actionId = getActionIdForStage(enabledAction, currentStage.value, index);
        if (!actionStates.value[actionId]) {
          actionStates.value[actionId] = initializeActionState(enabledAction);
        }
      });
    }
  });
}

function getActionId(action: Action, index?: number): string {
  return getActionIdForStage(action, currentStage.value, index);
}

function getActionKey(action: Action): string {
  const index = visibleActions.value.indexOf(action);
  return `${currentStage.value}-${index}-${getActionId(action)}`;
}

const visibleActions = computed(() => {
  const selectedActionIds = Object.entries(actionStates.value)
    .filter(([_, state]) => state.selected)
    .map(([id]) => id);

  const allActions: Action[] = [];
  const actionSources = new Map<Action, { enabledBy?: Action; actionIndex?: number }>();

  currentStageObj.value.actions.forEach((action, actionIndex) => {
    if (shouldShowAction(action)) {
      allActions.push(action);
      actionSources.set(action, { actionIndex });

      if (action.enablesActions) {
        action.enablesActions.forEach((enabledAction) => {
          if (shouldShowAction(enabledAction)) {
            allActions.push(enabledAction);
            actionSources.set(enabledAction, { enabledBy: action, actionIndex });
          }
        });
      }
    }
  });

  return allActions.filter((action) => {
    const source = actionSources.get(action);

    if (source?.enabledBy) {
      const enablerId = getActionId(source.enabledBy, source.actionIndex);
      if (!selectedActionIds.includes(enablerId)) {
        return false;
      }
    }

    const disabledByOthers = currentStageObj.value.actions.some((otherAction, otherIndex) => {
      const otherId = getActionId(otherAction, otherIndex);
      return (
        selectedActionIds.includes(otherId) &&
        otherAction.disablesActions?.includes(
          action.id || `action-${currentStage.value}-${source?.actionIndex}`,
        )
      );
    });

    return !disabledByOthers;
  });
});

const buttonActions = computed(() =>
  visibleActions.value.filter(
    (action) => action.type === "button" || action.type === "conditional-button",
  ),
);

const toggleActions = computed(() =>
  visibleActions.value.filter((action) => action.type === "toggle"),
);

const dropdownActions = computed(() =>
  visibleActions.value.filter((action) => action.type === "dropdown"),
);

const multiSelectActions = computed(() =>
  visibleActions.value.filter((action) => action.type === "multi-select-chips"),
);

function getDropdownValue(action: DropdownAction) {
  const actionId = getActionId(action);
  const visibleOptions = getVisibleDropdownOptions(action);
  const currentValue = actionStates.value[actionId]?.value ?? action.defaultOption ?? 0;

  const allOptions = action.options;
  const storedOption = allOptions[currentValue];

  if (storedOption && visibleOptions.includes(storedOption)) {
    return storedOption;
  }

  return visibleOptions[0] || null;
}

function isActionSelected(action: Action): boolean {
  const actionId = getActionId(action);
  return actionStates.value[actionId]?.selected || false;
}

function toggleAction(action: Action) {
  const actionId = getActionId(action);
  const state = actionStates.value[actionId];
  if (state) {
    state.selected = !state.selected;
    persistState();
  }
}

function selectDropdownOption(action: DropdownAction, selected: any) {
  const actionId = getActionId(action);
  const state = actionStates.value[actionId];
  if (state && selected !== undefined && selected !== null) {
    const optionIndex = action.options.findIndex(
      (opt) => opt === selected || (opt?.label && selected?.label && opt.label === selected.label),
    );

    if (optionIndex !== -1) {
      state.value = optionIndex;
      state.selected = true;
      persistState();
    }
  }
}

function isChipSelected(action: MultiSelectChipsAction, optionIndex: number): boolean {
  const actionId = getActionId(action);
  const selectedSet = actionStates.value[actionId]?.value as Set<number> | undefined;

  const visibleOptions = getVisibleMultiSelectOptions(action);
  const visibleOption = visibleOptions[optionIndex];
  const originalIndex = action.options.findIndex((opt) => opt === visibleOption);

  return selectedSet?.has(originalIndex) || false;
}

function toggleChip(action: MultiSelectChipsAction, optionIndex: number) {
  const actionId = getActionId(action);
  const state = actionStates.value[actionId];
  if (state && state.value instanceof Set) {
    const visibleOptions = getVisibleMultiSelectOptions(action);
    const visibleOption = visibleOptions[optionIndex];
    const originalIndex = action.options.findIndex((opt) => opt === visibleOption);

    if (originalIndex !== -1) {
      if (state.value.has(originalIndex)) {
        state.value.delete(originalIndex);
      } else {
        state.value.add(originalIndex);
      }
      state.selected = state.value.size > 0;
      persistState();
    }
  }
}

const isAnyVisibleInputs = computed(() => {
  return visibleActions.value.some((action) => {
    const visibleInputs = getVisibleInputs(action, actionStates.value);
    return visibleInputs.length > 0 && isActionSelected(action);
  });
});

async function assembleFullMessage() {
  const messageParts: MessagePart[] = [];

  for (let stageIndex = 0; stageIndex < checklist.length; stageIndex++) {
    const stage = checklist[stageIndex];

    await processStageActions(stage, stageIndex, messageParts);
  }

  messageParts.sort((a, b) => a.weight - b.weight);

  const finalMessage = expandVariables(
    messageParts
      .map((part) => part.content)
      .filter((content) => content.trim().length > 0)
      .join("\n\n"),
    props.project,
  );

  return finalMessage;
}

async function processStageActions(stage: Stage, stageIndex: number, messageParts: MessagePart[]) {
  const selectedActionIds = Object.entries(actionStates.value)
    .filter(([_, state]) => state.selected)
    .map(([id]) => id);

  for (let actionIndex = 0; actionIndex < stage.actions.length; actionIndex++) {
    const action = stage.actions[actionIndex];
    const actionId = getActionIdForStage(action, stageIndex, actionIndex);
    const state = actionStates.value[actionId];

    if (!state?.selected) continue;

    await processAction(action, actionId, state, selectedActionIds, stageIndex, messageParts);

    if (action.enablesActions) {
      for (let enabledIndex = 0; enabledIndex < action.enablesActions.length; enabledIndex++) {
        const enabledAction = action.enablesActions[enabledIndex];
        const enabledActionId = getActionIdForStage(
          enabledAction,
          stageIndex,
          actionIndex,
          enabledIndex,
        );
        const enabledState = actionStates.value[enabledActionId];

        if (enabledState?.selected) {
          await processAction(
            enabledAction,
            enabledActionId,
            enabledState,
            selectedActionIds,
            stageIndex,
            messageParts,
          );
        }
      }
    }
  }
}

async function processAction(
  action: Action,
  actionId: string,
  state: ActionState,
  selectedActionIds: string[],
  stageIndex: number,
  messageParts: MessagePart[],
) {
  const allValidActionIds: string[] = [];
  checklist.forEach((stage, stageIdx) => {
    stage.actions.forEach((stageAction, actionIdx) => {
      allValidActionIds.push(getActionIdForStage(stageAction, stageIdx, actionIdx));
      if (stageAction.enablesActions) {
        stageAction.enablesActions.forEach((enabledAction, enabledIdx) => {
          allValidActionIds.push(
            getActionIdForStage(enabledAction, stageIdx, actionIdx, enabledIdx),
          );
        });
      }
    });
  });

  if (action.type === "button" || action.type === "toggle") {
    const buttonAction = action as ButtonAction | ToggleAction;
    const message = await getActionMessage(buttonAction, selectedActionIds, allValidActionIds);
    if (message) {
      messageParts.push({
        weight: buttonAction.weight,
        content: processMessage(message, action, stageIndex, textInputValues.value),
        actionId,
        stageIndex,
      });
    }
  } else if (action.type === "conditional-button") {
    const conditionalAction = action as ConditionalButtonAction;
    const matchingVariant = findMatchingVariant(
      conditionalAction.messageVariants,
      selectedActionIds,
      allValidActionIds,
      stageIndex,
    );

    let message: string;
    let weight: number;

    if (matchingVariant) {
      message = (await matchingVariant.message()) as string;
      weight = matchingVariant.weight;
    } else if (conditionalAction.fallbackMessage) {
      message = (await conditionalAction.fallbackMessage()) as string;
      weight = conditionalAction.fallbackWeight ?? 0;
    } else {
      return;
    }

    messageParts.push({
      weight,
      content: processMessage(message, action, stageIndex, textInputValues.value),
      actionId,
      stageIndex,
    });
  } else if (action.type === "dropdown") {
    const dropdownAction = action as DropdownAction;
    const selectedIndex = state.value ?? 0;
    const selectedOption = dropdownAction.options[selectedIndex];

    if (selectedOption && "message" in selectedOption && "weight" in selectedOption) {
      const message = (await selectedOption.message()) as string;
      messageParts.push({
        weight: selectedOption.weight,
        content: processMessage(message, action, stageIndex, textInputValues.value),
        actionId,
        stageIndex,
      });
    }
  } else if (action.type === "multi-select-chips") {
    const multiSelectAction = action as MultiSelectChipsAction;
    const selectedIndices = state.value as Set<number>;

    for (const index of selectedIndices) {
      const option = multiSelectAction.options[index];
      if (option && "message" in option && "weight" in option) {
        const message = (await option.message()) as string;
        messageParts.push({
          weight: option.weight,
          content: processMessage(message, action, stageIndex, textInputValues.value),
          actionId: `${actionId}-option-${index}`,
          stageIndex,
        });
      }
    }
  }
}

function shouldShowStage(stage: Stage): boolean {
  if (typeof stage.shouldShow === "function") {
    return stage.shouldShow(props.project);
  }

  return true;
}

function shouldShowAction(action: Action): boolean {
  if (typeof action.shouldShow === "function") {
    return action.shouldShow(props.project);
  }

  return true;
}

function getVisibleDropdownOptions(action: DropdownAction) {
  return action.options.filter((option) => {
    if (typeof option.shouldShow === "function") {
      return option.shouldShow(props.project);
    }
    return true;
  });
}

function getVisibleMultiSelectOptions(action: MultiSelectChipsAction) {
  return action.options.filter((option) => {
    if (typeof option.shouldShow === "function") {
      return option.shouldShow(props.project);
    }
    return true;
  });
}

function shouldShowStageIndex(stageIndex: number): boolean {
  return shouldShowStage(checklist[stageIndex]);
}

function previousStage() {
  let targetStage = currentStage.value - 1;

  while (targetStage >= 0) {
    if (shouldShowStageIndex(targetStage)) {
      currentStage.value = targetStage;
      return;
    }
    targetStage--;
  }
}

function nextStage() {
  if (isModpackPermissionsStage.value && !modpackPermissionsComplete.value) {
    addNotification({
      title: "Modpack permissions stage unfinished",
      message: "Please complete the modpack permissions stage before proceeding.",
      type: "error",
    });

    return;
  }

  let targetStage = currentStage.value + 1;

  while (targetStage < checklist.length) {
    if (shouldShowStageIndex(targetStage)) {
      currentStage.value = targetStage;
      if (!isModpackPermissionsStage.value) {
        modpackPermissionsComplete.value = false;
      }

      return;
    }
    targetStage++;
  }
}

function goBackToStages() {
  generatedMessage.value = false;
  message.value = "";

  let targetStage = checklist.length - 1;
  while (targetStage >= 0) {
    if (shouldShowStageIndex(targetStage)) {
      currentStage.value = targetStage;
      return;
    }
    targetStage--;
  }
}

async function generateMessage() {
  if (loadingMessage.value) return;

  loadingMessage.value = true;

  router.push(`/${props.project.project_type}/${props.project.slug}/moderation`);

  try {
    const baseMessage = await assembleFullMessage();
    let fullMessage = baseMessage;

    if (
      props.project.project_type === "modpack" &&
      Object.keys(modpackJudgements.value).length > 0
    ) {
      const modpackMessage = generateModpackMessage(modpackJudgements.value);
      if (modpackMessage) {
        fullMessage = baseMessage ? `${baseMessage}\n\n${modpackMessage}` : modpackMessage;
      }
    }

    try {
      const formattedMessage = await prettier.format(fullMessage, {
        parser: "markdown",
        printWidth: 80,
        proseWrap: "always",
        tabWidth: 2,
        useTabs: false,
      });
      message.value = formattedMessage;
    } catch (formattingError) {
      console.warn("Failed to format markdown, using original:", formattingError);
      message.value = fullMessage;
    }

    generatedMessage.value = true;
  } catch (error) {
    console.error("Error generating message:", error);
    addNotification({
      title: "Error generating message",
      message: "Failed to generate moderation message. Please try again.",
      type: "error",
    });
  } finally {
    loadingMessage.value = false;
  }
}

function generateModpackMessage(judgements: ModerationJudgements) {
  const issues = [];

  const attributeMods = [];
  const noMods = [];
  const permanentNoMods = [];
  const unidentifiedMods = [];

  for (const [, judgement] of Object.entries(judgements)) {
    if (judgement.status === "with-attribution") {
      attributeMods.push(judgement.file_name);
    } else if (judgement.status === "no") {
      noMods.push(judgement.file_name);
    } else if (judgement.status === "permanent-no") {
      permanentNoMods.push(judgement.file_name);
    } else if (judgement.status === "unidentified") {
      unidentifiedMods.push(judgement.file_name);
    }
  }

  if (
    attributeMods.length > 0 ||
    noMods.length > 0 ||
    permanentNoMods.length > 0 ||
    unidentifiedMods.length > 0
  ) {
    issues.push("## Copyrighted content");

    if (attributeMods.length > 0) {
      issues.push(
        `${finalPermissionMessages["with-attribution"]}\n${attributeMods.map((mod) => `- ${mod}`).join("\n")}`,
      );
    }

    if (noMods.length > 0) {
      issues.push(`${finalPermissionMessages.no}\n${noMods.map((mod) => `- ${mod}`).join("\n")}`);
    }

    if (permanentNoMods.length > 0) {
      issues.push(
        `${finalPermissionMessages["permanent-no"]}\n${permanentNoMods.map((mod) => `- ${mod}`).join("\n")}`,
      );
    }

    if (unidentifiedMods.length > 0) {
      issues.push(
        `${finalPermissionMessages["unidentified"]}\n${unidentifiedMods.map((mod) => `- ${mod}`).join("\n")}`,
      );
    }
  }

  return issues.join("\n\n");
}

async function sendMessage(status: "approved" | "rejected" | "withheld") {
  try {
    await useBaseFetch(`project/${props.project.id}`, {
      method: "PATCH",
      body: {
        status,
      },
    });

    if (message.value) {
      await useBaseFetch(`thread/${props.project.thread_id}`, {
        method: "POST",
        body: {
          body: {
            type: "text",
            body: message.value,
          },
        },
      });
    }

    if (
      props.project.project_type === "modpack" &&
      Object.keys(modpackJudgements.value).length > 0
    ) {
      await useBaseFetch(`moderation/project`, {
        internal: true,
        method: "POST",
        body: modpackJudgements.value,
      });
    }

    done.value = true;

    // Clear local storage for future reviews
    localStorage.removeItem(`modpack-permissions-${props.project.id}`);
    localStorage.removeItem(`modpack-permissions-index-${props.project.id}`);
    localStorage.removeItem(`moderation-actions-${props.project.slug}`);
    localStorage.removeItem(`moderation-inputs-${props.project.slug}`);
    actionStates.value = {};

    addNotification({
      title: "Moderation submitted",
      message: `Project ${status} successfully.`,
      type: "success",
    });
  } catch (error) {
    console.error("Error submitting moderation:", error);
    addNotification({
      title: "Error submitting moderation",
      message: "Failed to submit moderation decision. Please try again.",
      type: "error",
    });
  }
}

async function goToNextProject() {
  const currentIds = JSON.parse(localStorage.getItem("moderation-future-projects") || "[]");

  if (currentIds.length === 0) {
    await navigateTo("/moderation/review");
    return;
  }

  const nextProjectId = currentIds[0];
  const remainingIds = currentIds.slice(1);

  localStorage.setItem("moderation-future-projects", JSON.stringify(remainingIds));

  await router.push({
    name: "type-id",
    params: {
      type: "project",
      id: nextProjectId,
    },
    state: {
      showChecklist: true,
    },
  });
}

async function exitModeration() {
  await navigateTo("/moderation/review");
}

const isLastVisibleStage = computed(() => {
  for (let i = currentStage.value + 1; i < checklist.length; i++) {
    if (shouldShowStageIndex(i)) {
      return false;
    }
  }
  return true;
});

const hasValidPreviousStage = computed(() => {
  for (let i = currentStage.value - 1; i >= 0; i--) {
    if (shouldShowStageIndex(i)) {
      return true;
    }
  }
  return false;
});

const stageOptions = computed<OverflowMenuOption[]>(() => {
  const options = checklist
    .map((stage, index) => {
      if (!shouldShowStage(stage)) return null;

      return {
        id: String(index),
        action: () => (currentStage.value = index),
        text: stage.id ? kebabToTitleCase(stage.id) : stage.title,
        color: index === currentStage.value && !generatedMessage.value ? "green" : undefined,
        hoverFilled: true,
        icon: stage.icon ? stage.icon : undefined,
      } as OverflowMenuOption;
    })
    .filter((opt): opt is OverflowMenuOption => opt !== null);

  options.push({
    id: "generate-message",
    action: () => generateMessage(),
    text: "Generate Message",
    color: generatedMessage.value ? "green" : undefined,
    hoverFilled: true,
    icon: CheckIcon,
  } as OverflowMenuOption);

  return options;
});
</script>

<style scoped lang="scss">
.moderation-checklist {
  @media (prefers-reduced-motion) {
    transition: none !important;
  }

  .button-actions-group,
  .toggle-actions-group,
  .dropdown-actions-group,
  .multi-select-actions-group {
    animation: fadeIn 0.2s ease-in-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(-5px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
}
</style>
