<template>
  <div
    class="moderation-checklist flex max-w-full flex-col rounded-2xl border-[1px] border-solid border-orange bg-bg-raised p-4 transition-all delay-200 duration-200 ease-in-out"
    :class="collapsed ? 'w-fit' : ''"
  >
    <div class="flex grow-0 items-center gap-2">
      <h1 class="m-0 mr-auto flex items-center gap-2 text-2xl font-extrabold text-contrast">
        <ScaleIcon class="text-orange" /> Moderation
      </h1>
      <ButtonStyled circular>
        <a v-tooltip="`Stage guidance`" target="_blank" :href="currentStageObj.guidance_url">
          <FileTextIcon />
        </a>
      </ButtonStyled>
      <ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
        <button v-tooltip="`Exit moderation`" @click="$emit('exit')">
          <XIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled circular>
        <button v-tooltip="collapsed ? `Expand` : `Collapse`" @click="$emit('toggleCollapsed')">
          <DropdownIcon class="transition-transform" :class="{ 'rotate-180': collapsed }" />
        </button>
      </ButtonStyled>
    </div>

    <Collapsible base-class="grow" class="flex grow flex-col" :collapsed="collapsed">
      <div class="my-4 h-[1px] w-full bg-divider" />
      <div class="flex-1">
        <h2 class="m-0 mb-2 text-lg font-extrabold">{{ currentStageObj.title }}</h2>
        <div class="options input-group">
          <button class="btn">Contains useless info</button>
          <button class="btn option-selected">Minecraft title</button>
          <button class="btn">Title similarities</button>
        </div>

        <div class="inputs universal-labels mt-4">
          <div>
            <label for="reason">
              <span class="label__title">
                Please elaborate on the issue
                <span class="required">*</span>
              </span>
            </label>
            <input id="reason" type="text" autocomplete="off" />
          </div>
        </div>
      </div>

      <!-- Stage control buttons -->
      <div class="mt-auto">
        <div
          class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-divider pt-4"
        >
          <div class="flex items-center gap-2">
            <ButtonStyled>
              <button>
                <XIcon aria-hidden="true" />
                Skip
              </button>
            </ButtonStyled>
          </div>

          <div class="flex items-center gap-2">
            <OverflowMenu :options="stageOptions" class="bg-transparent p-0">
              <ButtonStyled circular>
                <button v-tooltip="`Stages`">
                  <ListBulletedIcon />
                </button>
              </ButtonStyled>

              <template
                v-for="opt in stageOptions.filter((opt) => 'id' in opt && 'text' in opt)"
                #[`${opt.id}`]
                :key="opt.id"
              >
                {{ opt.text }}
              </template>
            </OverflowMenu>
            <ButtonStyled>
              <button :disabled="!(currentStage > 0)" @click="currentStage = currentStage - 1">
                <LeftArrowIcon aria-hidden="true" /> Previous
              </button>
            </ButtonStyled>
            <ButtonStyled color="brand">
              <button
                :disabled="!(currentStage < checklist.length - 1)"
                @click="currentStage = currentStage + 1"
              >
                <RightArrowIcon aria-hidden="true" /> Next
              </button>
            </ButtonStyled>
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
} from "@modrinth/assets";
import { checklist } from "@modrinth/moderation";
import { ButtonStyled, Collapsible, OverflowMenu, OverflowMenuOption } from "@modrinth/ui";
import type { Project } from "@modrinth/utils";
import { useLocalStorage } from "@vueuse/core";

const props = defineProps<{
  project: Project;
  collapsed: boolean;
}>();

const emit = defineEmits<{
  exit: [];
  toggleCollapsed: [];
}>();

const currentStageObj = computed(() => checklist[currentStage.value]);
const currentStage = useLocalStorage(`moderation-stage-${props.project.slug}`, () => 0);
const stageOptions = computed<OverflowMenuOption[]>(() => {
  return checklist.map((_stage, index) => {
    return {
      id: String(index),
      action: () => (currentStage.value = index),
      text: _stage.title,
      color: index === currentStage.value ? "green" : undefined,
      hoverFilled: true,
    } as OverflowMenuOption;
  });
});
</script>

<style scoped lang="scss">
.moderation-checklist {
  @media (prefers-reduced-motion) {
    transition: none !important;
  }

  .option-selected {
    color: var(--color-contrast);
    background-color: var(--color-brand-highlight);
    box-shadow:
      inset 0 0 0 transparent,
      0 0 0 2px var(--color-brand);
  }
}
</style>
