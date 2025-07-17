<template>
  <div class="universal-card flex h-24 items-center justify-between rounded-lg">
    <div class="flex items-center gap-3">
      <div class="flex-shrink-0 rounded-lg">
        <Avatar size="48px" :src="project.icon_url" />
      </div>

      <div class="flex flex-col">
        <h3 class="text-lg font-semibold">
          {{ project.title }}
        </h3>
        <nuxt-link
          target="_blank"
          v-if="enrichedProject.owner"
          class="flex items-center gap-1 align-middle text-sm hover:text-brand"
          :to="`/user/${enrichedProject.owner.user.username}`"
        >
          <Avatar
            :src="enrichedProject.owner.user.avatar_url"
            circle
            size="16px"
            class="inline-block"
          />
          <span>{{ enrichedProject.owner.user.username }}</span>
        </nuxt-link>
        <nuxt-link
          target="_blank"
          v-else-if="enrichedProject.org"
          class="flex items-center gap-1 align-middle text-sm hover:text-brand"
          :to="`/organization/${enrichedProject.org.slug}`"
        >
          <Avatar :src="enrichedProject.org.icon_url" circle size="16px" class="inline-block" />
          <span>{{ enrichedProject.org.name }}</span>
        </nuxt-link>
      </div>
    </div>

    <div class="flex items-center gap-4">
      <span class="flex items-center gap-1 text-sm">
        <BoxIcon class="size-4" aria-hidden="true" v-if="enrichedProject.project_type === 'mod'" />
        <PaintbrushIcon
          class="size-4"
          aria-hidden="true"
          v-else-if="enrichedProject.project_type === 'resourcepack'"
        />
        <BracesIcon
          class="size-4"
          aria-hidden="true"
          v-else-if="enrichedProject.project_type === 'datapack'"
        />
        <PackageOpenIcon
          class="size-4"
          aria-hidden="true"
          v-else-if="enrichedProject.project_type === 'modpack'"
        />
        <GlassesIcon
          class="size-4"
          aria-hidden="true"
          v-else-if="enrichedProject.project_type === 'shader'"
        />
        <PlugIcon
          class="size-4"
          aria-hidden="true"
          v-else-if="enrichedProject.project_type === 'plugin'"
        />
        {{ formatProjectType(project.project_type) }} &#x2022;
        <span
          :class="{
            'text-red': daysInQueue > 4,
            'text-orange': daysInQueue > 2,
          }"
          v-tooltip="`Since ${queuedDate.toLocaleString()}`"
          >{{ getSubmittedTime(project) }}</span
        >
      </span>

      <div class="flex items-center gap-2">
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
import { useModerationStore } from "~/store/moderation";

const formatRelativeTime = useRelativeTime();
const moderationStore = useModerationStore();
const router = useRouter();

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
  return dayjs(enrichedProject.value.queued).toDate();
});

const daysInQueue = computed(() => {
  return getDaysQueued(queuedDate.value);
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
  const date = project.queued || project.published || project.created;
  if (!date) return "Unknown";

  try {
    return `Submitted ${formatRelativeTime(dayjs(date).toISOString())}`;
  } catch (error) {
    return "Unknown";
  }
}
</script>
