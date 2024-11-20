<template>
  <div v-if="showInvitation" class="universal-card information invited">
    <h2>邀请加入资源</h2>
    <p>
      您已被邀请成为该项目的成员，权限为 '{{ currentMember.role }}'.
    </p>
    <div class="input-group">
      <button class="iconified-button brand-button" @click="acceptInvite()">
        <CheckIcon />接收
      </button>
      <button class="iconified-button danger-button" @click="declineInvite()">
        <CrossIcon />拒绝
      </button>
    </div>
  </div>
  <div
    v-if="
      currentMember &&
      nags.filter((x) => x.condition).length > 0 &&
      (project.status === 'draft' || tags.rejectedStatuses.includes(project.status))
    "
    class="author-actions universal-card mb-4"
  >
    <div class="header__row">
      <div class="header__title">
        <h2>发布前检查</h2>
        <div class="checklist">
          <span class="checklist__title">进度:</span>
          <div class="checklist__items">
            <div
              v-for="nag in nags"
              :key="`checklist-${nag.id}`"
              v-tooltip="nag.title"
              :aria-label="nag.title"
              class="circle"
              :class="'circle ' + (!nag.condition ? 'done' : '') + nag.status"
            >
              <CheckIcon v-if="!nag.condition" />
              <RequiredIcon v-else-if="nag.status === 'required'" />
              <SuggestionIcon v-else-if="nag.status === 'suggestion'" />
              <ModerationIcon v-else-if="nag.status === 'review'" />
            </div>
          </div>
        </div>
      </div>
      <div class="input-group">
        <button
          class="square-button"
          :class="{ 'not-collapsed': !collapsed }"
          @click="toggleCollapsed()"
        >
          <DropdownIcon />
        </button>
      </div>
    </div>
    <div v-if="!collapsed" class="grid-display width-16">
      <div
        v-for="nag in nags.filter((x) => x.condition && !x.hide)"
        :key="nag.id"
        class="grid-display__item"
      >
        <span class="label">
          <RequiredIcon
            v-if="nag.status === 'required'"
            v-tooltip="'Required'"
            aria-label="Required"
            :class="nag.status"
          />
          <SuggestionIcon
            v-else-if="nag.status === 'suggestion'"
            v-tooltip="'Suggestion'"
            aria-label="Suggestion"
            :class="nag.status"
          />
          <ModerationIcon
            v-else-if="nag.status === 'review'"
            v-tooltip="'Review'"
            aria-label="Review"
            :class="nag.status"
          />{{ nag.title }}</span
        >
        {{ nag.description }}
        <NuxtLink
          v-if="nag.link"
          :class="{ invisible: nag.link.hide }"
          class="goto-link"
          :to="`/${project.project_type}/${project.slug ? project.slug : project.id}/${
            nag.link.path
          }`"
        >
          {{ nag.link.title }}
          <ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
        </NuxtLink>
        <button
          v-else-if="nag.action"
          class="btn btn-orange"
          :disabled="nag.action.disabled()"
          @click="nag.action.onClick"
        >
          <SendIcon />
          {{ nag.action.title }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { formatProjectType } from "~/plugins/shorthands.js";

import ChevronRightIcon from "~/assets/images/utils/chevron-right.svg?component";
import DropdownIcon from "~/assets/images/utils/dropdown.svg?component";
import CheckIcon from "~/assets/images/utils/check.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import RequiredIcon from "~/assets/images/utils/asterisk.svg?component";
import SuggestionIcon from "~/assets/images/utils/lightbulb.svg?component";
import ModerationIcon from "~/assets/images/sidebar/admin.svg?component";
import SendIcon from "~/assets/images/utils/send.svg?component";
import { acceptTeamInvite, removeTeamMember } from "~/helpers/teams.js";

const props = defineProps({
  project: {
    type: Object,
    required: true,
  },
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
  currentMember: {
    type: Object,
    default: null,
  },
  allMembers: {
    type: Object,
    default: null,
  },
  isSettings: {
    type: Boolean,
    default: false,
  },
  collapsed: {
    type: Boolean,
    default: false,
  },
  routeName: {
    type: String,
    default: "",
  },
  auth: {
    type: Object,
    required: true,
  },
  tags: {
    type: Object,
    required: true,
  },
  setProcessing: {
    type: Function,
    default() {
      return () => {
        addNotification({
          group: "main",
          title: "发生错误",
          text: "setProcessing function not found",
          type: "error",
        });
      };
    },
  },
  toggleCollapsed: {
    type: Function,
    default() {
      return () => {
        addNotification({
          group: "main",
          title: "发生错误",
          text: "未找到 toggleCollapsed 函数",
          type: "error",
        });
      };
    },
  },
  updateMembers: {
    type: Function,
    default() {
      return () => {
        addNotification({
          group: "main",
          title: "发生错误",
          text: "updateMembers 函数未找到",
          type: "error",
        });
      };
    },
  },
});

const featuredGalleryImage = computed(() => props.project.gallery.find((img) => img.featured));

const nags = computed(() => [
  {
    condition: props.versions.length < 1,
    title: "上传版本",
    id: "upload-version",
    description: "资源至少需要一个版本才能提交审核。",
    status: "required",
    link: {
      path: "versions",
      title: "访问版本页面",
      hide: props.routeName === "type-id-versions",
    },
  },
  {
    condition:
      props.project.body === "" || props.project.body.startsWith("# Placeholder description"),
    title: "介绍",
    id: "add-description",
    description:
      "需要清晰的介绍资源和基本功能，请尽量丰富一些.",
    status: "required",
    link: {
      path: "settings/description",
      title: "访问介绍页面",
      hide: props.routeName === "type-id-settings-description",
    },
  },
  {
    condition: !props.project.icon_url,
    title: "设置图标",
    id: "add-icon",
    description:
      "设置一个一眼就能记住的LOGO图标.",
    status: "suggestion",
    link: {
      path: "settings",
      title: "访问图标页面",
      hide: props.routeName === "type-id-settings",
    },
  },
  {
    condition: props.project.gallery.length === 0 || !featuredGalleryImage,
    title: "渲染图",
    id: "feature-gallery-image",
    description: "设置一个精致的渲染图,一张大图被用于宣传推广",
    status: "suggestion",
    link: {
      path: "gallery",
      title: "访问渲染图页面",
      hide: props.routeName === "type-id-gallery",
    },
  },
  {
    hide: props.project.versions.length === 0,
    condition: props.project.categories.length < 1,
    title: "选择标签",
    id: "select-tags",
    description: "选择适合你资源的标签",
    status: "suggestion",
    link: {
      path: "settings/tags",
      title: "访问标签设置页面",
      hide: props.routeName === "type-id-settings-tags",
    },
  },
  {
    condition: !(
      props.project.issues_url ||
      props.project.source_url ||
      props.project.wiki_url ||
      props.project.discord_url ||
      props.project.donation_urls.length > 0
    ),
    title: "更多URL",
    id: "add-links",
    description:
      "BUG反馈地址,开源地址等本网站之外的第三方网站链接",
    status: "suggestion",
    link: {
      path: "settings/links",
      title: "访问URL设置页面",
      hide: props.routeName === "type-id-settings-links",
    },
  },
  {
    hide:
      props.project.versions.length === 0 ||
      props.project.project_type === "resourcepack" ||
      props.project.project_type === "plugin" ||
      props.project.project_type === "shader" ||
      props.project.project_type === "datapack",
    condition:
      props.project.client_side === "unknown" ||
      props.project.server_side === "unknown" ||
      (props.project.client_side === "unsupported" && props.project.server_side === "unsupported"),
    title: "运行环境",
    id: "select-environments",
    description: `选择资源 ${formatProjectType(
      props.project.project_type,
    ).toLowerCase()} 适用于服务端还是客户端`,
    status: "required",
    link: {
      path: "settings",
      title: "访问常规设置页面",
      hide: props.routeName === "type-id-settings",
    },
  },
  {
    condition: props.project.license.id === "LicenseRef-Unknown",
    title: "选择许可证",
    id: "select-license",
    description: `选择您 ${formatProjectType(
      props.project.project_type,
    ).toLowerCase()} 所遵循的许可证.`,
    status: "required",
    link: {
      path: "settings/license",
      title: "访问许可证页面",
      hide: props.routeName === "type-id-settings-license",
    },
  },
  {
    condition: props.project.status === "draft",
    title: "提交审核",
    id: "submit-for-review",
    description:
      "您的项目暂时仅供项目成员查看,必须经过版主审核才能发布.",
    status: "review",
    link: null,
    action: {
      onClick: submitForReview,
      title: "提交审核.",
      disabled: () => nags.value.filter((x) => x.condition && x.status === "required").length > 0,
    },
  },
  {
    condition: props.tags.rejectedStatuses.includes(props.project.status),
    title: "重新提交审核",
    id: "resubmit-for-review",
    description: `您的项目被版主设置为 ${props.project.status} . 您可以在回复后重新提交审核`,
    status: "review",
    link: {
      path: "moderation",
      title: "访问审核页面",
      hide: props.routeName === "type-id-moderation",
    },
  },
]);

const showInvitation = computed(() => {
  if (props.allMembers && props.auth) {
    const member = props.allMembers.find((x) => x.user.id === props.auth.user.id);
    return member && !member.accepted;
  }
  return false;
});

const acceptInvite = () => {
  acceptTeamInvite(props.project.team);
  props.updateMembers();
};

const declineInvite = () => {
  removeTeamMember(props.project.team, props.auth.user.id);
  props.updateMembers();
};

const submitForReview = async () => {
  if (
    !props.acknowledgedMessage ||
    nags.value.filter((x) => x.condition && x.status === "required").length === 0
  ) {
    await props.setProcessing();
  }
};
</script>

<style lang="scss" scoped>
.invited {
}

.author-actions {
  margin-top: var(--spacing-card-md);

  &:empty {
    display: none;
  }

  .invisible {
    visibility: hidden;
  }

  .header__row {
    align-items: center;
    column-gap: var(--spacing-card-lg);
    row-gap: var(--spacing-card-md);
    max-width: 100%;

    .header__title {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      column-gap: var(--spacing-card-lg);
      row-gap: var(--spacing-card-md);
      flex-basis: min-content;

      h2 {
        margin: 0 auto 0 0;
      }
    }

    button {
      svg {
        transition: transform 0.25s ease-in-out;
      }

      &.not-collapsed svg {
        transform: rotate(180deg);
      }
    }
  }

  .grid-display__item .label {
    display: flex;
    gap: var(--spacing-card-xs);
    align-items: center;

    .required {
      color: var(--color-red);
    }

    .suggestion {
      color: var(--color-purple);
    }

    .review {
      color: var(--color-orange);
    }
  }

  .checklist {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: var(--spacing-card-xs);
    width: fit-content;
    flex-wrap: wrap;
    max-width: 100%;

    .checklist__title {
      font-weight: bold;
      margin-right: var(--spacing-card-xs);
      color: var(--color-text-dark);
    }

    .checklist__items {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: var(--spacing-card-xs);
      width: fit-content;
      max-width: 100%;
    }

    .circle {
      --circle-size: 2rem;
      --background-color: var(--color-bg);
      --content-color: var(--color-gray);
      width: var(--circle-size);
      height: var(--circle-size);
      border-radius: 50%;
      background-color: var(--background-color);
      display: flex;
      justify-content: center;
      align-items: center;

      svg {
        color: var(--content-color);
        width: calc(var(--circle-size) / 2);
        height: calc(var(--circle-size) / 2);
      }

      &.required {
        --content-color: var(--color-red);
      }

      &.suggestion {
        --content-color: var(--color-purple);
      }

      &.review {
        --content-color: var(--color-orange);
      }

      &.done {
        --background-color: var(--color-green);
        --content-color: var(--color-brand-inverted);
      }
    }
  }
}
</style>
