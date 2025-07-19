<template>
  <div
    class="universal-card flex min-h-[6rem] flex-col justify-between gap-3 rounded-lg p-4 sm:h-24 sm:flex-row sm:items-center sm:gap-0"
  >
    <div class="flex min-w-0 flex-1 items-center gap-3">
      <div class="flex-shrink-0 rounded-lg">
        <Avatar size="48px" :src="project.icon_url" />
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <h3 class="truncate text-lg font-semibold">
          {{ project.title }}
        </h3>
        <nuxt-link
          v-if="enrichedProject.owner"
          target="_blank"
          class="flex items-center gap-1 truncate align-middle text-sm hover:text-brand"
          :to="`/user/${enrichedProject.owner.user.username}`"
        >
          <Avatar
            :src="enrichedProject.owner.user.avatar_url"
            circle
            size="16px"
            class="inline-block flex-shrink-0"
          />
          <span class="truncate">{{ enrichedProject.owner.user.username }}</span>
        </nuxt-link>
        <nuxt-link
          v-else-if="enrichedProject.org"
          target="_blank"
          class="flex items-center gap-1 truncate align-middle text-sm hover:text-brand"
          :to="`/organization/${enrichedProject.org.slug}`"
        >
          <Avatar
            :src="enrichedProject.org.icon_url"
            circle
            size="16px"
            class="inline-block flex-shrink-0"
          />
          <span class="truncate">{{ enrichedProject.org.name }}</span>
        </nuxt-link>
      </div>
    </div>

    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-4">
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-1">
        <span class="flex items-center gap-1 whitespace-nowrap text-sm">
          <BoxIcon
            v-if="enrichedProject.project_type === 'mod'"
            class="size-4 flex-shrink-0"
            aria-hidden="true"
          />
          <PaintbrushIcon
            v-else-if="enrichedProject.project_type === 'resourcepack'"
            class="size-4 flex-shrink-0"
            aria-hidden="true"
          />
          <BracesIcon
            v-else-if="enrichedProject.project_type === 'datapack'"
            class="size-4 flex-shrink-0"
            aria-hidden="true"
          />
          <PackageOpenIcon
            v-else-if="enrichedProject.project_type === 'modpack'"
            class="size-4 flex-shrink-0"
            aria-hidden="true"
          />
          <GlassesIcon
            v-else-if="enrichedProject.project_type === 'shader'"
            class="size-4 flex-shrink-0"
            aria-hidden="true"
          />
          <PlugIcon
            v-else-if="enrichedProject.project_type === 'plugin'"
            class="size-4 flex-shrink-0"
            aria-hidden="true"
          />
          <span class="hidden sm:inline">{{ formatProjectType(project.project_type) }}</span>
          <span class="sm:hidden">{{
            formatProjectType(project.project_type).substring(0, 3)
          }}</span>
        </span>

        <span class="hidden text-sm sm:inline">&#x2022;</span>

        <span
          v-tooltip="`Since ${queuedDate.toLocaleString()}`"
          class="truncate text-sm"
          :class="{
            'text-red': daysInQueue > 4,
            'text-orange': daysInQueue > 2,
          }"
        >
          <span class="hidden sm:inline">{{ getSubmittedTime(project) }}</span>
          <span class="sm:hidden">{{ getSubmittedTime(project).replace("Submitted ", "") }}</span>
        </span>
      </div>

      <div class="flex items-center justify-end gap-2 sm:justify-start">
        <ButtonStyled circular>
          <NuxtLink target="_blank" :to="`/project/${project.slug}`">
            <EyeIcon class="size-4" />
          </NuxtLink>
        </ButtonStyled>
        <ButtonStyled circular color="orange" @click="openProjectForReview">
          <button>
            <ScaleIcon class="size-4" />
          </button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import dayjs from "dayjs";
import {
  EyeIcon,
  PaintbrushIcon,
  ScaleIcon,
  BoxIcon,
  GlassesIcon,
  PlugIcon,
  PackageOpenIcon,
  BracesIcon,
} from "@modrinth/assets";
import { useRelativeTime, Avatar, ButtonStyled } from "@modrinth/ui";
import { formatProjectType, type Organization, type TeamMember } from "@modrinth/utils";
import { computed } from "vue";
import { useModerationStore } from "~/store/moderation.ts";

const formatRelativeTime = useRelativeTime();
const moderationStore = useModerationStore();

const props = defineProps<{
  project: any;
  owner?: TeamMember | null;
  org?: Organization | null;
}>();

const enrichedProject = computed(() => ({
  ...props.project,
  owner: props.owner,
  org: props.org,
}));

function getDaysQueued(date: Date): number {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  return Math.floor(diff / (1000 * 60 * 60 * 24));
}

const queuedDate = computed(() => {
  return dayjs(
    enrichedProject.value.queued || enrichedProject.value.created || enrichedProject.value.updated,
  );
});

const daysInQueue = computed(() => {
  return getDaysQueued(queuedDate.value.toDate());
});

function openProjectForReview() {
  moderationStore.setSingleProject(props.project.id);
  navigateTo({
    name: "type-id",
    params: {
      type: "project",
      id: props.project.id,
    },
    state: {
      showChecklist: true,
    },
  });
}

function getSubmittedTime(project: any): string {
  const date = project.queued || project.created || project.updated;
  if (!date) return "Unknown";

  try {
    return `Submitted ${formatRelativeTime(dayjs(date).toISOString())}`;
  } catch {
    return "Unknown";
  }
}
</script>
