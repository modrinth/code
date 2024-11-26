<template>
  <div>
    <section class="universal-card">
      <h2>资源状态</h2>
      <Badge :type="project.status" />
      <p v-if="isApproved(project)">
        您发布的资源已通过审核 您可以点击
        <router-link :to="`${getProjectLink(project)}/settings`" class="text-link"
          >资源设置</router-link
        >
        去修改资源是否公开
      </p>
      <div v-else-if="isUnderReview(project)">
        <p>
          BBSMC 内容版主团队会努力审核所有已提交的项目。 通常，新项目将在 24 到 48
          小时内审核完毕。请记住，较大的项目（尤其是模组包）可能需要更多时间审核。
          某些节假日或活动也可能导致延迟，具体取决于版主的空闲时间。 如果 BBSMC
          版主对您有任何问题或疑虑，他们会在下面留言。
        </p>
      </div>
      <template v-else-if="isRejected(project)">
        <p>
          您的项目目前不符合 BBSMC 要求
          <nuxt-link to="/legal2/rules" class="text-link" target="_blank">内容规则</nuxt-link>
          版主已要求您进行更改才能批准。请阅读以下版主的留言，并在重新提交前解决他们的意见。
        </p>
        <p class="warning"><IssuesIcon /> 重复提交而不回应版主的留言回复可能会导致帐户被封禁。</p>
      </template>
      <h3>当前可见性</h3>
      <ul class="visibility-info">
        <li v-if="isListed(project)">
          <CheckIcon class="good" />
          搜索结果中显示
        </li>
        <li v-else>
          <ExitIcon class="bad" />
          不可被搜索
        </li>
        <li v-if="isListed(project)">
          <CheckIcon class="good" />
          在个人资料中显示
        </li>
        <li v-else>
          <ExitIcon class="bad" />
          不可在个人资料中显示
        </li>
        <li v-if="isPrivate(project)">
          <ExitIcon class="bad" />
          无法通过直接链接访问
        </li>
        <li v-else>
          <CheckIcon class="good" />
          可直接通过链接打开查看
        </li>
      </ul>
    </section>
    <section id="messages" class="universal-card">
      <h2>消息</h2>
      <p>
        这是与 BBSMC
        版主的私人对话页面,他们可能会向您发送有关此项目的问题的消息.仅当您提交资源以供审核时才会显示此对话。
      </p>
      <ConversationThread
        v-if="thread"
        :thread="thread"
        :project="project"
        :set-status="setStatus"
        :current-member="currentMember"
        :auth="auth"
        @update-thread="(newThread) => (thread = newThread)"
      />
    </section>
  </div>
</template>
<script setup>
import { ExitIcon, CheckIcon, IssuesIcon } from "@modrinth/assets";
import { Badge } from "@modrinth/ui";
import ConversationThread from "~/components/ui/thread/ConversationThread.vue";
import {
  getProjectLink,
  isApproved,
  isListed,
  isPrivate,
  isRejected,
  isUnderReview,
} from "~/helpers/projects.js";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
  resetProject: {
    type: Function,
    required: true,
    default: () => {},
  },
});

const app = useNuxtApp();
const auth = await useAuth();

const { data: thread } = await useAsyncData(`thread/${props.project.thread_id}`, () =>
  useBaseFetch(`thread/${props.project.thread_id}`),
);
async function setStatus(status) {
  startLoading();

  try {
    const data = {};
    data.status = status;
    await useBaseFetch(`project/${props.project.id}`, {
      method: "PATCH",
      body: data,
    });

    const project = props.project;
    project.status = status;
    await props.resetProject();
    thread.value = await useBaseFetch(`thread/${thread.value.id}`);
  } catch (err) {
    app.$notify({
      group: "main",
      title: "发生错误",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }

  stopLoading();
}
</script>
<style lang="scss" scoped>
.stacked {
  display: flex;
  flex-direction: column;
}

.status-message {
  :deep(.badge) {
    display: contents;

    svg {
      vertical-align: top;
      margin: 0;
    }
  }

  p:last-child {
    margin-bottom: 0;
  }
}

.unavailable-error {
  .code {
    margin-top: var(--spacing-card-sm);
  }

  svg {
    vertical-align: top;
  }
}

.visibility-info {
  padding: 0;
  list-style: none;

  li {
    display: flex;
    align-items: center;
    gap: var(--spacing-card-xs);
  }
}

svg {
  &.good {
    color: var(--color-green);
  }

  &.bad {
    color: var(--color-red);
  }
}

.warning {
  color: var(--color-orange);
  font-weight: bold;
}
</style>
