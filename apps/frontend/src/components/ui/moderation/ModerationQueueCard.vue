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
        <div class="flex items-center gap-2 text-sm">
          <template v-if="project.owner">
            <Avatar :src="project.owner.user.avatar_url" circle size="16px" class="inline-block" />
            <span>{{ project.owner.user.username }}</span>
          </template>
          <template v-else-if="project.org">
            <Avatar :src="project.org.icon_url" circle size="16px" class="inline-block" />
            <span>{{ project.org.name }}</span>
          </template>
          <template v-else>
            <div class="h-4 w-4 rounded-full bg-gray-300"></div>
            <span>Unknown</span>
          </template>

          <span class="bg-blue-100 text-blue-800 rounded px-2 py-0.5 text-xs">
            {{
              formatProjectType(
                project.inferred_project_type || project.project_types?.[0] || "unknown",
              )
            }}
          </span>
        </div>
      </div>
    </div>

    <div class="flex items-center gap-4">
      <span class="text-sm">
        {{ getSubmittedTime(project) }}
      </span>

      <div class="flex items-center gap-2">
        <ButtonStyled circular>
          <button>
            <EyeIcon class="size-4" />
          </button>
        </ButtonStyled>

        <ButtonStyled circular color="orange">
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
import { EyeIcon, ScaleIcon } from "@modrinth/assets";
import { useRelativeTime, Avatar, ButtonStyled } from "@modrinth/ui";
import { formatProjectType } from "@modrinth/utils";

const formatRelativeTime = useRelativeTime();

const props = defineProps<{ project: any }>();

console.log(props.project);

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
