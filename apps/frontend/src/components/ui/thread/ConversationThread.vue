<template>
  <div>
    <Modal
      ref="modalSubmit"
      :header="isRejected(project) ? 'Resubmit for review' : 'Submit for review'"
    >
      <div class="modal-submit universal-body">
        <span>
          You're submitting <span class="project-title">{{ project.title }}</span> to be reviewed
          again by the moderators.
        </span>
        <span>
          Make sure you have addressed the comments from the moderation team.
          <span class="known-errors">
            Repeated submissions without addressing the moderators' comments may result in an
            account suspension.
          </span>
        </span>
        <Checkbox
          v-model="submissionConfirmation"
          description="Confirm I have addressed the messages from the moderators"
        >
          I confirm that I have properly addressed the moderators' comments.
        </Checkbox>
        <div class="input-group push-right">
          <button
            class="iconified-button moderation-button"
            :disabled="!submissionConfirmation"
            @click="resubmit()"
          >
            <ScaleIcon aria-hidden="true" />
            Resubmit for review
          </button>
        </div>
      </div>
    </Modal>
    <Modal ref="modalReply" header="Reply to thread">
      <div class="modal-submit universal-body">
        <span>
          Your project is already approved. As such, the moderation team does not actively monitor
          this thread. However, they may still see your message if there is a problem with your
          project.
        </span>
        <span>
          If you need to get in contact with the moderation team, please use the
          <a class="text-link" href="https://support.modrinth.com" target="_blank">
            Modrinth Help Center
          </a>
          and click the green bubble to contact support.
        </span>
        <Checkbox
          v-model="replyConfirmation"
          description="Confirm moderators do not actively monitor this"
        >
          I acknowledge that the moderators do not actively monitor the thread.
        </Checkbox>
        <div class="input-group push-right">
          <button
            class="btn btn-primary"
            :disabled="!replyConfirmation"
            @click="sendReplyFromModal()"
          >
            <ReplyIcon aria-hidden="true" />
            Reply to thread
          </button>
        </div>
      </div>
    </Modal>
    <div v-if="flags.developerMode" class="thread-id">
      Thread ID:
      <CopyCode :text="thread.id" />
    </div>
    <div v-if="sortedMessages.length > 0" class="messages universal-card recessed">
      <ThreadMessage
        v-for="message in sortedMessages"
        :key="'message-' + message.id"
        :thread="thread"
        :message="message"
        :members="members"
        :report="report"
        :auth="auth"
        raised
        @update-thread="() => updateThreadLocal()"
      />
    </div>
    <template v-if="report && report.closed">
      <p>This thread is closed and new messages cannot be sent to it.</p>
      <button v-if="isStaff(auth.user)" class="iconified-button" @click="reopenReport()">
        <CheckCircleIcon aria-hidden="true" />
        Reopen thread
      </button>
    </template>
    <template v-else-if="!report || !report.closed">
      <div class="markdown-editor-spacing">
        <MarkdownEditor
          v-model="replyBody"
          :placeholder="sortedMessages.length > 0 ? 'Reply to thread...' : 'Send a message...'"
          :on-image-upload="onUploadImage"
        />
      </div>
      <div class="input-group">
        <button
          v-if="sortedMessages.length > 0"
          class="btn btn-primary"
          :disabled="!replyBody"
          @click="isApproved(project) && !isStaff(auth.user) ? openReplyModal() : sendReply()"
        >
          <ReplyIcon aria-hidden="true" />
          Reply
        </button>
        <button
          v-else
          class="btn btn-primary"
          :disabled="!replyBody"
          @click="isApproved(project) && !isStaff(auth.user) ? openReplyModal() : sendReply()"
        >
          <SendIcon aria-hidden="true" />
          Send
        </button>
        <button
          v-if="isStaff(auth.user)"
          class="btn"
          :disabled="!replyBody"
          @click="sendReply(null, true)"
        >
          <ScaleIcon aria-hidden="true" />
          Add private note
        </button>
        <template v-if="currentMember && !isStaff(auth.user)">
          <template v-if="isRejected(project)">
            <button
              v-if="replyBody"
              class="iconified-button moderation-button"
              @click="openResubmitModal(true)"
            >
              <ScaleIcon aria-hidden="true" />
              Resubmit for review with reply
            </button>
            <button
              v-else
              class="iconified-button moderation-button"
              @click="openResubmitModal(false)"
            >
              <ScaleIcon aria-hidden="true" />
              Resubmit for review
            </button>
          </template>
        </template>
        <div class="spacer"></div>
        <div class="input-group extra-options">
          <template v-if="report">
            <template v-if="isStaff(auth.user)">
              <button
                v-if="replyBody"
                class="iconified-button danger-button"
                @click="closeReport(true)"
              >
                <CheckCircleIcon aria-hidden="true" />
                Close with reply
              </button>
              <button v-else class="iconified-button danger-button" @click="closeReport()">
                <CheckCircleIcon aria-hidden="true" />
                Close thread
              </button>
            </template>
          </template>
          <template v-if="project">
            <template v-if="isStaff(auth.user)">
              <button
                v-if="replyBody"
                class="btn btn-green"
                :disabled="isApproved(project)"
                @click="sendReply(requestedStatus)"
              >
                <CheckIcon aria-hidden="true" />
                Approve with reply
              </button>
              <button
                v-else
                class="btn btn-green"
                :disabled="isApproved(project)"
                @click="setStatus(requestedStatus)"
              >
                <CheckIcon aria-hidden="true" />
                Approve
              </button>
              <div class="joined-buttons">
                <button
                  v-if="replyBody"
                  class="btn btn-danger"
                  :disabled="project.status === 'rejected'"
                  @click="sendReply('rejected')"
                >
                  <XIcon aria-hidden="true" />
                  Reject with reply
                </button>
                <button
                  v-else
                  class="btn btn-danger"
                  :disabled="project.status === 'rejected'"
                  @click="setStatus('rejected')"
                >
                  <XIcon aria-hidden="true" />
                  Reject
                </button>
                <OverflowMenu
                  class="btn btn-danger btn-dropdown-animation icon-only"
                  :options="
                    replyBody
                      ? [
                          {
                            id: 'withhold-reply',
                            color: 'danger',
                            action: () => {
                              sendReply('withheld');
                            },
                            hoverFilled: true,
                            disabled: project.status === 'withheld',
                          },
                        ]
                      : [
                          {
                            id: 'withhold',
                            color: 'danger',
                            action: () => {
                              setStatus('withheld');
                            },
                            hoverFilled: true,
                            disabled: project.status === 'withheld',
                          },
                        ]
                  "
                >
                  <DropdownIcon style="rotate: 180deg" aria-hidden="true" />
                  <template #withhold-reply>
                    <EyeOffIcon aria-hidden="true" />
                    Withhold with reply
                  </template>
                  <template #withhold>
                    <EyeOffIcon aria-hidden="true" />
                    Withhold
                  </template>
                </OverflowMenu>
              </div>
            </template>
          </template>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { CopyCode, OverflowMenu, MarkdownEditor } from "@modrinth/ui";
import {
  DropdownIcon,
  ReplyIcon,
  SendIcon,
  CheckCircleIcon,
  XIcon,
  EyeOffIcon,
  CheckIcon,
  ScaleIcon,
} from "@modrinth/assets";
import { useImageUpload } from "~/composables/image-upload.ts";
import ThreadMessage from "~/components/ui/thread/ThreadMessage.vue";
import { isStaff } from "~/helpers/users.js";
import { isApproved, isRejected } from "~/helpers/projects.js";
import Modal from "~/components/ui/Modal.vue";
import Checkbox from "~/components/ui/Checkbox.vue";

const props = defineProps({
  thread: {
    type: Object,
    required: true,
  },
  report: {
    type: Object,
    required: false,
    default: null,
  },
  project: {
    type: Object,
    required: false,
    default: null,
  },
  setStatus: {
    type: Function,
    required: false,
    default: () => {},
  },
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
  auth: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(["update-thread"]);

const app = useNuxtApp();
const flags = useFeatureFlags();

const members = computed(() => {
  const members = {};
  for (const member of props.thread.members) {
    members[member.id] = member;
  }
  return members;
});

const replyBody = ref("");

const sortedMessages = computed(() => {
  if (props.thread !== null) {
    return props.thread.messages
      .slice()
      .sort((a, b) => app.$dayjs(a.created) - app.$dayjs(b.created));
  }
  return [];
});

const modalSubmit = ref(null);
const modalReply = ref(null);

async function updateThreadLocal() {
  let threadId = null;
  if (props.project) {
    threadId = props.project.thread_id;
  } else if (props.report) {
    threadId = props.report.thread_id;
  }
  let thread = null;
  if (threadId) {
    thread = await useBaseFetch(`thread/${threadId}`);
  }
  emit("update-thread", thread);
}

const imageIDs = ref([]);

async function onUploadImage(file) {
  const response = await useImageUpload(file, { context: "thread_message" });

  imageIDs.value.push(response.id);
  // Keep the last 10 entries of image IDs
  imageIDs.value = imageIDs.value.slice(-10);

  return response.url;
}

async function sendReplyFromModal(status = null, privateMessage = false) {
  modalReply.value.hide();
  await sendReply(status, privateMessage);
}

async function sendReply(status = null, privateMessage = false) {
  try {
    const body = {
      body: {
        type: "text",
        body: replyBody.value,
        private: privateMessage,
      },
    };

    if (imageIDs.value.length > 0) {
      body.body = {
        ...body.body,
        uploaded_images: imageIDs.value,
      };
    }

    await useBaseFetch(`thread/${props.thread.id}`, {
      method: "POST",
      body,
    });

    replyBody.value = "";

    await updateThreadLocal();
    if (status !== null) {
      props.setStatus(status);
    }
  } catch (err) {
    app.$notify({
      group: "main",
      title: "Error sending message",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

async function closeReport(reply) {
  if (reply) {
    await sendReply();
  }

  try {
    await useBaseFetch(`report/${props.report.id}`, {
      method: "PATCH",
      body: {
        closed: true,
      },
    });
    await updateThreadLocal();
  } catch (err) {
    app.$notify({
      group: "main",
      title: "Error closing report",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

async function reopenReport() {
  try {
    await useBaseFetch(`report/${props.report.id}`, {
      method: "PATCH",
      body: {
        closed: false,
      },
    });
    await updateThreadLocal();
  } catch (err) {
    app.$notify({
      group: "main",
      title: "Error reopening report",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

const replyWithSubmission = ref(false);
const submissionConfirmation = ref(false);
const replyConfirmation = ref(false);

function openResubmitModal(reply) {
  submissionConfirmation.value = false;
  replyWithSubmission.value = reply;
  modalSubmit.value.show();
}

function openReplyModal(reply) {
  replyConfirmation.value = false;
  modalReply.value.show();
}

async function resubmit() {
  if (replyWithSubmission.value) {
    await sendReply("processing");
  } else {
    await props.setStatus("processing");
  }
  modalSubmit.value.hide();
}

const requestedStatus = computed(() => props.project.requested_status ?? "approved");
</script>

<style lang="scss" scoped>
.markdown-editor-spacing {
  margin-bottom: var(--gap-md);
}

.messages {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-card-md);
}

.resizable-textarea-wrapper {
  margin-bottom: var(--spacing-card-sm);

  textarea {
    padding: var(--spacing-card-bg);
    width: 100%;
  }

  .chips {
    margin-bottom: var(--spacing-card-md);
  }

  .preview {
    overflow-y: auto;
  }
}

.thread-id {
  margin-bottom: var(--spacing-card-md);
  font-weight: bold;
  color: var(--color-heading);
}

.input-group {
  .spacer {
    flex-grow: 1;
    flex-shrink: 1;
  }

  .extra-options {
    flex-basis: fit-content;
  }
}

.modal-submit {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-lg);

  .project-title {
    font-weight: bold;
  }
}
</style>
