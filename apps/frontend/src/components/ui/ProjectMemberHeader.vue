<template>
  <div v-if="showInvitation" class="universal-card information invited">
    <h2>Invitation to join project</h2>
    <p v-if="currentMember?.project_role">
      You've been invited be a member of this project with the role of '{{
        currentMember.project_role
      }}'.
    </p>
    <p v-else>You've been invited to join this project. Please accept or decline the invitation.</p>
    <div class="input-group">
      <button class="iconified-button brand-button" @click="acceptInvite()">
        <CheckIcon />
        Accept
      </button>
      <button class="iconified-button danger-button" @click="declineInvite()">
        <XIcon />
        Decline
      </button>
    </div>
  </div>
  <div
    v-if="
      currentMember &&
      visibleNags.length > 0 &&
      (project.status === 'draft' || tags.rejectedStatuses.includes(project.status))
    "
    class="universal-card mb-4"
  >
    <div class="flex max-w-full flex-wrap items-center gap-x-6 gap-y-4">
      <div class="flex flex-auto flex-wrap items-center gap-x-6 gap-y-4">
        <h2 class="my-0 mr-auto">Publishing checklist</h2>
        <div class="flex w-fit max-w-full flex-row flex-wrap items-center gap-2">
          <span class="mr-2 font-bold text-dark">Progress:</span>
          <div class="flex w-fit max-w-full flex-row items-center gap-2">
            <div
              v-for="nag in applicableNags"
              :key="`checklist-${nag.id}`"
              v-tooltip="nag.title"
              :aria-label="nag.title"
              :class="[
                'flex h-8 w-8 items-center justify-center rounded-full transition-colors',
                isNagComplete(nag)
                  ? 'bg-green text-inverted'
                  : nag.status === 'required'
                    ? 'bg-bg text-red'
                    : nag.status === 'warning'
                      ? 'bg-bg text-orange'
                      : 'bg-bg text-purple',
              ]"
            >
              <CheckIcon v-if="isNagComplete(nag)" class="h-4 w-4" />
              <component :is="nag.icon || getDefaultIcon(nag.status)" v-else class="h-4 w-4" />
            </div>
          </div>
        </div>
      </div>
      <div class="input-group">
        <button
          :class="['square-button', !collapsed && '[&>svg]:rotate-180']"
          @click="toggleCollapsed()"
        >
          <DropdownIcon class="duration-250 transition-transform ease-in-out" />
        </button>
      </div>
    </div>
    <div v-if="!collapsed" class="grid-display width-16">
      <div v-for="nag in visibleNags" :key="nag.id" class="grid-display__item">
        <span class="flex items-center gap-2">
          <component
            :is="nag.icon || getDefaultIcon(nag.status)"
            v-tooltip="getStatusTooltip(nag.status)"
            :class="[
              'h-4 w-4',
              nag.status === 'required' && 'text-red',
              nag.status === 'warning' && 'text-orange',
              nag.status === 'suggestion' && 'text-purple',
            ]"
            :aria-label="getStatusTooltip(nag.status)"
          />
          {{ nag.title }}
        </span>
        {{ nag.description(nagContext) }}
        <NuxtLink
          v-if="nag.link && shouldShowLink(nag)"
          :to="`/${project.project_type}/${project.slug ? project.slug : project.id}/${
            nag.link.path
          }`"
          class="goto-link"
        >
          {{ nag.link.title }}
          <ChevronRightIcon aria-hidden="true" class="featured-header-chevron" />
        </NuxtLink>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ChevronRightIcon,
  CheckIcon,
  XIcon,
  AsteriskIcon,
  LightBulbIcon,
  TriangleAlertIcon,
  DropdownIcon,
} from "@modrinth/assets";
import { acceptTeamInvite, removeTeamMember } from "~/helpers/teams.js";
import { nags } from "@modrinth/moderation";
import type { Nag, NagContext, NagStatus } from "@modrinth/moderation";
import type { Project, User, Version } from "@modrinth/utils";
import type { Component } from "vue";

interface Tags {
  rejectedStatuses: string[];
}

interface Auth {
  user: {
    id: string;
  };
}

interface Member {
  accepted?: boolean;
  project_role?: string;
  user?: Partial<User>;
}

interface Props {
  project: Project;
  versions?: Version[];
  currentMember?: Member | null;
  allMembers?: Member[] | null;
  isSettings?: boolean;
  collapsed?: boolean;
  routeName?: string;
  auth: Auth;
  tags: Tags;
  setProcessing?: (processing: boolean) => void;
  toggleCollapsed?: () => void;
  updateMembers?: () => void | Promise<void>;
}

const props = withDefaults(defineProps<Props>(), {
  versions: () => [],
  currentMember: null,
  allMembers: null,
  isSettings: false,
  collapsed: false,
  routeName: "",
});

const emit = defineEmits<{
  toggleCollapsed: [];
  updateMembers: [];
  setProcessing: [processing: boolean];
}>();

const nagContext = computed<NagContext>(() => ({
  project: props.project,
  versions: props.versions,
  currentMember: props.currentMember as User,
  currentRoute: props.routeName,
}));

const applicableNags = computed<Nag[]>(() => {
  return nags.filter((nag) => {
    return !nag.shouldShow || nag.shouldShow(nagContext.value);
  });
});

const isNagComplete = (nag: Nag): boolean => {
  const context = nagContext.value;
};

const visibleNags = computed<Nag[]>(() => {
  return applicableNags.value.filter((nag) => !isNagComplete(nag));
});

const shouldShowLink = (nag: Nag): boolean => {
  if (!nag.link) return false;
  if (!nag.link.shouldShow) return true;
  return nag.link.shouldShow(nagContext.value);
};

const getDefaultIcon = (status: NagStatus): Component => {
  switch (status) {
    case "required":
      return AsteriskIcon;
    case "warning":
      return TriangleAlertIcon;
    case "suggestion":
      return LightBulbIcon;
    default:
      return AsteriskIcon;
  }
};

const getStatusTooltip = (status: NagStatus): string => {
  switch (status) {
    case "required":
      return "Required";
    case "warning":
      return "Warning";
    case "suggestion":
      return "Suggestion";
    default:
      return "Required";
  }
};

const showInvitation = computed<boolean>(() => {
  if (props.allMembers && props.auth) {
    const member = props.allMembers.find((x) => x.user.id === props.auth.user.id);
    return !!member && !member.accepted;
  }
  return false;
});

const toggleCollapsed = (): void => {
  if (props.toggleCollapsed) {
    props.toggleCollapsed();
  } else {
    emit("toggleCollapsed");
  }
};

const updateMembers = async (): Promise<void> => {
  if (props.updateMembers) {
    await props.updateMembers();
  } else {
    emit("updateMembers");
  }
};

const setProcessing = (processing: boolean): void => {
  if (props.setProcessing) {
    props.setProcessing(processing);
  } else {
    emit("setProcessing", processing);
  }
};

const acceptInvite = async (): Promise<void> => {
  try {
    setProcessing(true);
    await acceptTeamInvite(props.project.team);
    await updateMembers();
    addNotification({
      group: "main",
      title: "Success",
      text: "You have joined the project team",
      type: "success",
    });
  } catch (error) {
    addNotification({
      group: "main",
      title: "Error",
      text: "Failed to accept team invitation",
      type: "error",
    });
  } finally {
    setProcessing(false);
  }
};

const declineInvite = async (): Promise<void> => {
  try {
    setProcessing(true);
    await removeTeamMember(props.project.team, props.auth.user.id);
    await updateMembers();
    addNotification({
      group: "main",
      title: "Success",
      text: "You have declined the team invitation",
      type: "success",
    });
  } catch (error) {
    addNotification({
      group: "main",
      title: "Error",
      text: "Failed to decline team invitation",
      type: "error",
    });
  } finally {
    setProcessing(false);
  }
};
</script>

<style lang="scss" scoped>
.duration-250 {
  transition-duration: 250ms;
}
</style>
