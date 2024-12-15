<script setup>
import {
  ClipboardCopyIcon,
  LinkIcon,
  ShareIcon,
  MailIcon,
  GlobeIcon,
  TwitterIcon,
  MastodonIcon,
  RedditIcon,
} from '@modrinth/assets'
import { computed, ref, nextTick } from 'vue'
import QrcodeVue from 'qrcode.vue'
import { Button, Modal } from '../index'

const props = defineProps({
  header: {
    type: String,
    default: 'Share',
  },
  shareTitle: {
    type: String,
    default: 'Modrinth',
  },
  shareText: {
    type: String,
    default: null,
  },
  link: {
    type: Boolean,
    default: false,
  },
  openInNewTab: {
    type: Boolean,
    default: true,
  },
  noblur: {
    type: Boolean,
    default: false,
  },
  onHide: {
    type: Function,
    default() {
      return () => {}
    },
  },
})

const shareModal = ref(null)

const qrCode = ref(null)
const qrImage = ref(null)
const content = ref(null)
const url = ref(null)
const canShare = ref(false)
const share = () => {
  navigator.share(
    props.link
      ? {
          title: props.shareTitle.toString(),
          text: props.shareText,
          url: url.value,
        }
      : {
          title: props.shareTitle.toString(),
          text: content.value,
        },
  )
}

const show = async (passedContent) => {
  content.value = props.shareText ? `${props.shareText}\n\n${passedContent}` : passedContent
  shareModal.value.show()
  if (props.link) {
    url.value = passedContent
    nextTick(() => {
      console.log(qrCode.value)
      fetch(qrCode.value.getElementsByTagName('canvas')[0].toDataURL('image/png'))
        .then((res) => res.blob())
        .then((blob) => {
          console.log(blob)
          qrImage.value = blob
        })
    })
  }
  if (navigator.canShare({ title: props.shareTitle.toString(), text: content.value })) {
    canShare.value = true
  }
}

const copyImage = async () => {
  const item = new ClipboardItem({ 'image/png': qrImage.value })
  await navigator.clipboard.write([item])
}

const copyText = async () => {
  await navigator.clipboard.writeText(url.value ?? content.value)
}

const sendEmail = computed(
  () =>
    `mailto:user@test.com
    ?subject=${encodeURIComponent(props.shareTitle)}
    &body=${encodeURIComponent(content.value)}`,
)

const targetParameter = computed(() => (props.openInNewTab ? '_blank' : '_self'))

const sendTweet = computed(
  () => `https://twitter.com/intent/tweet?text=${encodeURIComponent(content.value)}`,
)

const sendToot = computed(() => `https://tootpick.org/#text=${encodeURIComponent(content.value)}`)

const postOnReddit = computed(
  () =>
    `https://www.reddit.com/submit?title=${encodeURIComponent(props.shareTitle)}&text=${encodeURIComponent(
      content.value,
    )}`,
)

defineExpose({
  show,
})
</script>

<template>
  <Modal ref="shareModal" :header="header" :noblur="noblur" :on-hide="onHide">
    <div class="share-body">
      <div v-if="link" class="qr-wrapper">
        <div ref="qrCode">
          <QrcodeVue :value="url" class="qr-code" margin="3" />
        </div>
        <Button
          v-tooltip="'Copy QR code'"
          icon-only
          class="copy-button"
          aria-label="Copy QR code"
          @click="copyImage"
        >
          <ClipboardCopyIcon aria-hidden="true" />
        </Button>
      </div>
      <div v-else class="resizable-textarea-wrapper">
        <textarea v-model="content" />
        <Button
          v-tooltip="'Copy Text'"
          icon-only
          aria-label="Copy Text"
          class="copy-button transparent"
          @click="copyText"
        >
          <ClipboardCopyIcon aria-hidden="true" />
        </Button>
      </div>
      <div class="all-buttons">
        <div v-if="link" class="iconified-input">
          <LinkIcon />
          <input type="text" :value="url" readonly />
          <Button v-tooltip="'Copy Text'" aria-label="Copy Text" class="r-btn" @click="copyText">
            <ClipboardCopyIcon aria-hidden="true" />
          </Button>
        </div>
        <div class="button-row">
          <Button v-if="canShare" v-tooltip="'Share'" aria-label="Share" icon-only @click="share">
            <ShareIcon aria-hidden="true" />
          </Button>
          <a
            v-tooltip="'Send as an email'"
            class="btn icon-only"
            :href="sendEmail"
            :target="targetParameter"
            aria-label="Send as an email"
          >
            <MailIcon aria-hidden="true" />
          </a>
          <a
            v-if="link"
            v-tooltip="'Open link in browser'"
            class="btn icon-only"
            :target="targetParameter"
            :href="url"
            aria-label="Open link in browser"
          >
            <GlobeIcon aria-hidden="true" />
          </a>
          <a
            v-tooltip="'Toot about it'"
            class="btn mastodon icon-only"
            :target="targetParameter"
            :href="sendToot"
            aria-label="Toot about it"
          >
            <MastodonIcon aria-hidden="true" />
          </a>
          <a
            v-tooltip="'Tweet about it'"
            class="btn twitter icon-only"
            :target="targetParameter"
            :href="sendTweet"
            aria-label="Tweet about it"
          >
            <TwitterIcon aria-hidden="true" />
          </a>
          <a
            v-tooltip="'Share on Reddit'"
            class="btn reddit icon-only"
            :target="targetParameter"
            :href="postOnReddit"
            aria-label="Share on Reddit"
          >
            <RedditIcon aria-hidden="true" />
          </a>
        </div>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.share-body {
  display: flex;
  flex-direction: row;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--gap-sm);
  padding: var(--gap-lg);
}

.all-buttons {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  flex-grow: 1;
  justify-content: center;
}

.iconified-input {
  width: 100%;

  input {
    flex-basis: auto;
  }
}

.button-row {
  display: flex;
  flex-direction: row;
  gap: var(--gap-sm);

  .btn {
    fill: var(--color-contrast);
    color: var(--color-contrast);

    &.reddit {
      background-color: #ff4500;
    }

    &.mastodon {
      background-color: #563acc;
    }

    &.twitter {
      background-color: #1da1f2;
    }
  }
}

.qr-wrapper {
  position: relative;
  margin: 0 auto;

  &:hover {
    .copy-button {
      opacity: 1;
    }
  }
}

.qr-code {
  background-color: white !important;
  border-radius: var(--radius-md);
}

.copy-button {
  position: absolute;
  top: 0;
  right: 0;
  margin: var(--gap-sm);
  transition: all 0.2s ease-in-out;
  opacity: 0;

  @media (prefers-reduced-motion) {
    transition: none !important;
  }
}

.resizable-textarea-wrapper {
  position: relative;
  height: 100%;

  textarea {
    width: 100%;
    margin: 0;
  }

  .btn {
    opacity: 1;
    margin: 0;
  }
}
</style>
