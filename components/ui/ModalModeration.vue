<template>
  <Modal ref="modal" header="Project moderation">
    <div v-if="project !== null" class="moderation-modal universal-body">
      <p>
        A moderation message is optional, but it can be used to communicate problems with a
        project's team members. The body is also optional and supports markdown formatting!
      </p>
      <div v-if="status" class="status">
        <span>New project status: </span>
        <Badge :type="status" />
      </div>
      <h3>Message title</h3>
      <input v-model="moderationMessage" type="text" placeholder="Enter the message..." />
      <h3>Message body</h3>
      <div class="textarea-wrapper">
        <Chips v-model="bodyViewMode" class="separator" :items="['source', 'preview']" />
        <textarea
          v-if="bodyViewMode === 'source'"
          id="body"
          v-model="moderationMessageBody"
          :disabled="!moderationMessage"
          :placeholder="
            moderationMessage
              ? 'Type a body to your moderation message here...'
              : 'You must add a title before you add a body.'
          "
        />
        <div v-else class="markdown-body preview" v-html="renderString(moderationMessageBody)" />
      </div>
      <div class="push-right input-group">
        <button
          v-if="moderationMessage || moderationMessageBody"
          class="iconified-button"
          @click="
            () => {
              moderationMessage = ''
              moderationMessageBody = ''
            }
          "
        >
          <TrashIcon />
          Clear message
        </button>
        <button class="iconified-button" @click="$refs.modal.hide()">
          <CrossIcon />
          Cancel
        </button>
        <button class="iconified-button brand-button" @click="saveProject">
          <CheckIcon />
          Confirm
        </button>
      </div>
    </div>
  </Modal>
</template>

<script>
import TrashIcon from '~/assets/images/utils/trash.svg'
import CrossIcon from '~/assets/images/utils/x.svg'
import Modal from '~/components/ui/Modal.vue'
import Chips from '~/components/ui/Chips.vue'
import Badge from '~/components/ui/Badge.vue'
import CheckIcon from '~/assets/images/utils/check.svg'
import { renderString } from '~/helpers/parse.js'

export default {
  components: {
    TrashIcon,
    CrossIcon,
    CheckIcon,
    Modal,
    Chips,
    Badge,
  },
  props: {
    project: {
      type: Object,
      default: null,
    },
    status: {
      type: String,
      default: null,
    },
    onClose: {
      type: Function,
      default: null,
    },
  },
  data() {
    return {
      bodyViewMode: 'source',
      moderationMessage:
        this.project && this.project.moderation_message ? this.project.moderation_message : '',
      moderationMessageBody:
        this.project && this.project.moderation_message_body
          ? this.project.moderation_message_body
          : '',
    }
  },
  methods: {
    renderString,
    async saveProject() {
      startLoading()

      try {
        const data = {
          moderation_message: this.moderationMessage ? this.moderationMessage : null,
          moderation_message_body: this.moderationMessageBody ? this.moderationMessageBody : null,
        }
        if (this.status) {
          data.status = this.status
        }
        await useBaseFetch(`project/${this.project.id}`, {
          method: 'PATCH',
          body: data,
          ...this.$defaultHeaders(),
        })

        this.$refs.modal.hide()
        if (this.onClose !== null) {
          this.onClose()
        }
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data ? err.data.description : err,
          type: 'error',
        })
      }

      stopLoading()
    },
    show() {
      this.$refs.modal.show()
      this.moderationMessage =
        this.project && this.project.moderator_message && this.project.moderator_message.message
          ? this.project.moderator_message.message
          : ''
      this.moderationMessageBody =
        this.project && this.project.moderator_message && this.project.moderator_message.body
          ? this.project.moderator_message.body
          : ''
    },
  },
}
</script>

<style scoped lang="scss">
.moderation-modal {
  padding: var(--spacing-card-lg);

  .status {
    display: flex;
    align-items: center;
    margin-bottom: 0.5rem;

    span {
      margin-right: 0.5rem;
    }
  }

  .textarea-wrapper {
    margin-top: 0.5rem;
    height: 15rem;

    .preview {
      overflow-y: auto;
    }
  }

  .separator {
    margin: var(--spacing-card-sm) 0;
  }
}
</style>
