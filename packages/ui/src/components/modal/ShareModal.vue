<script setup>
import {
  ClipboardCopyIcon,
  LinkIcon,
  ShareIcon,
  MailIcon,
  GlobeIcon,
  MastodonIcon,
  RedditIcon,
  BlueskyIcon,
} from '@modrinth/assets'
import { computed, ref, nextTick } from 'vue'
import QrcodeVue from 'qrcode.vue'
import { Button } from '../index'
import NewModal from './NewModal.vue'

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
    default: '',
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
const content = ref('')
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
  if (passedContent) {
    content.value = props.shareText ? `${props.shareText}\n\n${passedContent}` : passedContent
  } else {
    content.value = props.shareText ? props.shareText : ''
  }
  shareModal.value.show()
  if (props.link) {
    url.value = passedContent
    nextTick(() => {
      //console.log(qrCode.value)
      fetch(qrCode.value.getElementsByTagName('canvas')[0].toDataURL('image/png'))
        .then((res) => res.blob())
        .then((blob) => {
          //console.log(blob)
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

// eslint-disable-next-line no-unused-vars
const sendTweet = computed(() => `https://www.youtube.com/watch?v=dQw4w9WgXcQ`)

const sendToot = computed(() => `https://tootpick.org/#text=${encodeURIComponent(content.value)}`)

const postOnReddit = computed(
  () =>
    `https://www.reddit.com/submit?title=${encodeURIComponent(props.shareTitle)}&text=${encodeURIComponent(
      content.value,
    )}`,
)

const postOnBluesky = computed(
  () => `https://bsky.app/intent/compose?text=${encodeURIComponent(content.value)}`,
)

defineExpose({
  show,
})
</script>

<template>
  <NewModal ref="shareModal" :header="header" :on-hide="onHide">
    <div class="flex items-center flex-wrap gap-2 p-lg">
      <div class="flex flex-col">
        <span v-if="!link" class="text-lg font-semibold text-contrast"> Content </span>
        <span class="text-xs"> Markdown is not supported. </span>
      </div>
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
      <div class="flex flex-col gap-2 grow justify-center">
        <div v-if="link" class="iconified-input">
          <LinkIcon />
          <input type="text" :value="url" readonly />
          <Button v-tooltip="'Copy Text'" aria-label="Copy Text" class="r-btn" @click="copyText">
            <ClipboardCopyIcon aria-hidden="true" />
          </Button>
        </div>
        <div class="flex gap-2 items-center flex-wrap">
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
            v-tooltip="'Post about it on Bluesky'"
            class="btn bsky icon-only"
            :target="targetParameter"
            :href="postOnBluesky"
            aria-label="Post about it on Bluesky"
          >
            <BlueskyIcon aria-hidden="true" />
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
  </NewModal>
</template>

<style scoped lang="scss">
.iconified-input {
  width: 100%;

  input {
    flex-basis: auto;
  }
}

.btn {
  fill: var(--color-contrast);
  color: var(--color-contrast);

  &.reddit {
    background-color: #ff4500;
  }

  &.mastodon {
    background-color: #563acc;
  }

  &.bsky {
    background-color: #0085ff;
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
