<template>
  <div>
    <section class="universal-card">
      <h2>Project status</h2>
      <Badge :type="project.status" />
      <p v-if="isApproved(project)">
        Your project been approved by the moderators and you may freely change project visibility in
        <router-link :to="`${getProjectLink(project)}/settings`" class="text-link"
          >your project's settings</router-link
        >.
      </p>
      <p v-else-if="isUnderReview(project)">
        Project reviews typically take 24 to 48 hours and they will leave a message below if they
        have any questions or concerns for you. If your review has taken more than 48 hours, check
        our Discord or social media for moderation delays.
      </p>
      <template v-else-if="isRejected(project)">
        <p>
          Your project does not currently meet Modrinth's
          <nuxt-link to="/legal/rules" class="text-link" target="_blank">content rules</nuxt-link>
          and the moderators have requested you make changes before it can be approved. Read the
          messages from the moderators below and address their comments before resubmitting.
        </p>
        <p class="warning">
          Repeated submissions without addressing the moderators' comments may result in an account
          suspension.
        </p>
      </template>
      <h3>Current visibility</h3>
      <ul class="visibility-info">
        <li v-if="isListed(project)">
          <CheckIcon class="good" />
          Listed in search results
        </li>
        <li v-else>
          <ExitIcon class="bad" />
          Not listed in search results
        </li>
        <li v-if="isListed(project)">
          <CheckIcon class="good" />
          Listed on the profiles of members
        </li>
        <li v-else>
          <ExitIcon class="bad" />
          Not listed on the profiles of members
        </li>
        <li v-if="isPrivate(project)">
          <ExitIcon class="bad" />
          Not accessible with a direct link
        </li>
        <li v-else>
          <CheckIcon class="good" />
          Accessible with a direct link
        </li>
      </ul>
    </section>
    <section id="messages" class="universal-card">
      <h2>Messages</h2>
      <p>
        This is a private conversation thread with the Modrinth moderators. They will message you
        for issues concerning your project on Modrinth, and you are welcome to message them about
        things concerning your project.
      </p>
      <ConversationThread
        v-if="thread"
        :thread="thread"
        :update-thread="(newThread) => (thread = newThread)"
        :project="project"
        :set-status="setStatus"
        :current-member="currentMember"
      />
    </section>
  </div>
</template>
<script setup>
import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import Badge from '~/components/ui/Badge.vue'
import {
  getProjectLink,
  isApproved,
  isListed,
  isPrivate,
  isRejected,
  isUnderReview,
} from '~/helpers/projects.js'
import ExitIcon from 'assets/images/utils/x.svg'
import CheckIcon from 'assets/images/utils/check.svg'

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {}
    },
  },
  currentMember: {
    type: Object,
    default() {
      return null
    },
  },
})

const emit = defineEmits(['update:project'])

const app = useNuxtApp()

const { data: thread } = await useAsyncData(`thread/${props.project.thread_id}`, () =>
  useBaseFetch(`thread/${props.project.thread_id}`, app.$defaultHeaders())
)
async function setStatus(status) {
  startLoading()

  try {
    const data = {}
    data.status = status
    await useBaseFetch(`project/${props.project.id}`, {
      method: 'PATCH',
      body: data,
      ...app.$defaultHeaders(),
    })
    const project = props.project
    project.status = status
    emit('update:project', project)
    thread.value = await useBaseFetch(`thread/${thread.value.id}`, app.$defaultHeaders())
  } catch (err) {
    app.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }

  stopLoading()
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
    color: var(--color-brand-green);
  }

  &.bad {
    color: var(--color-special-red);
  }
}

.warning {
  color: var(--color-special-orange);
}
</style>
