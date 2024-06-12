<template>
  <div>
    <section class="universal-card">
      <h2>Project status</h2>
      <Badge :type="project.status" />
      <p v-if="isApproved(project)">
        Your project has been approved by the moderators and you may freely change project
        visibility in
        <router-link :to="`${getProjectLink(project)}/settings`" class="text-link"
          >your project's settings</router-link
        >.
      </p>
      <div v-else-if="isUnderReview(project)">
        <p>
          Modrinth's team of content moderators work hard to review all submitted projects.
          Typically, you can expect a new project to be reviewed within 24 to 48 hours. Please keep
          in mind that larger projects, especially modpacks, may require more time to review.
          Certain holidays or events may also lead to delays depending on moderator availability.
          Modrinth's moderators will leave a message below if they have any questions or concerns
          for you.
        </p>
        <p>
          If your review has taken more than 48 hours, check our
          <a
            class="text-link"
            href="https://support.modrinth.com/en/articles/8793355-modrinth-project-review-times"
            target="_blank"
          >
            support article on review times
          </a>
          for moderation delays.
        </p>
      </div>
      <template v-else-if="isRejected(project)">
        <p>
          Your project does not currently meet Modrinth's
          <nuxt-link to="/legal/rules" class="text-link" target="_blank">content rules</nuxt-link>
          and the moderators have requested you make changes before it can be approved. Read the
          messages from the moderators below and address their comments before resubmitting.
        </p>
        <p class="warning">
          <IssuesIcon /> Repeated submissions without addressing the moderators' comments may result
          in an account suspension.
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
        This is a private conversation thread with the Modrinth moderators. They may message you
        with issues concerning this project. This thread is only checked when you submit your
        project for review. For additional inquiries, contact
        <a href="https://support.modrinth.com">Modrinth support</a>.
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
import { Badge, ExitIcon, CheckIcon, IssuesIcon } from 'omorphia'
import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import {
  getProjectLink,
  isApproved,
  isListed,
  isPrivate,
  isRejected,
  isUnderReview,
} from '~/helpers/projects.js'

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
  resetProject: {
    type: Function,
    required: true,
    default: () => {},
  },
})

const app = useNuxtApp()
const auth = await useAuth()

const { data: thread } = await useAsyncData(`thread/${props.project.thread_id}`, () =>
  useBaseFetch(`thread/${props.project.thread_id}`)
)
async function setStatus(status) {
  startLoading()

  try {
    const data = {}
    data.status = status
    await useBaseFetch(`project/${props.project.id}`, {
      method: 'PATCH',
      body: data,
    })

    const project = props.project
    project.status = status
    await props.resetProject()
    thread.value = await useBaseFetch(`thread/${thread.value.id}`)
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
    color: var(--color-red);
  }
}

.warning {
  color: var(--color-orange);
  font-weight: bold;
}
</style>
