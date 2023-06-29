<script setup>
import {Button, Modal, ClipboardCopyIcon, Avatar, LinkIcon, SendIcon, GlobeIcon} from 'omorphia'
import {defineProps, ref} from 'vue'
import {ofetch} from "ofetch";
import {TwitterIcon, MastodonIcon, RedditIcon} from "@/assets/external";
import {openLink} from "@/helpers/utils.js";

const props = defineProps({
  header: {
    type: String,
    required: true,
  },
  shareTitle: {
    type: String,
    default: 'Share',
  },
  shareText: {
    type: String,
    default: 'Share',
  },
  link: {
    type: Boolean,
    default: false,
  }
})

const shareModal = ref(null);

const qrCode = ref(null);
const qrImage = ref(null);
const content = ref(null);

const show = async (passedContent) => {
  content.value = passedContent
  const url = 'https://api.qrserver.com/v1/create-qr-code/?size=150x150&data=' + encodeURIComponent(passedContent);
  const response = await ofetch(url);
  qrImage.value = response;
  qrCode.value = URL.createObjectURL(response);
  shareModal.value.show()
}

const copyImage = async () => {
  const item = new ClipboardItem({ 'image/png': qrImage.value });
  await navigator.clipboard.write([item]);
}

const copyText = async () => {
  await navigator.clipboard.writeText(content.value);
}

const sendEmail = () => {
  openLink(`mailto:user@example.com?subject=${encodeURIComponent(props.shareTitle)}&body=` + encodeURIComponent(`${props.shareText}\n${content.value}`))
}

const sendTweet = () => {
  openLink(`https://twitter.com/intent/tweet?text=` + encodeURIComponent(`${props.shareText}\n${content.value}`))
}

const sendToot = () => {
  openLink(`https://mastodon.social/share?text=` + encodeURIComponent(`${props.shareText}\n${content.value}`))
}

const postOnReddit = () => {
  openLink(`https://www.reddit.com/submit?title=${encodeURIComponent(props.shareTitle)}&url=` + encodeURIComponent(`${props.shareText}\n${content.value}`))
}

defineExpose({
  show,
})
</script>

<template>
  <Modal ref="shareModal" :header="header">
    <div class="share-body">
      <div v-if="link" class="qr-wrapper">
        <Avatar :src="qrCode" alt="QR Code" class="qr-code" size="md" />
        <Button v-tooltip="'Copy QR code'" icon-only class="copy-button" @click="copyImage">
          <ClipboardCopyIcon/>
        </Button>
      </div>
      <div class="all-buttons">
        <div v-if="link" class="iconified-input">
          <LinkIcon />
          <input
            type="text"
            :value="content"
            readonly
            disabled
            />
          <Button v-tooltip="'Copy Text'" @click="copyText">
            <ClipboardCopyIcon />
          </Button>
        </div>
        <div v-else class="resizable-textarea-wrapper">
          <textarea
            v-model="content"
          />
        </div>
        <div class="button-row">
          <Button v-if="!link" v-tooltip="'Copy Text'" icon-only @click="copyText">
            <ClipboardCopyIcon />
          </Button>
          <Button v-tooltip="'Send as an email'" icon-only @click="sendEmail">
            <SendIcon />
          </Button>
          <Button v-if="link" v-tooltip="'Open link in browser'" icon-only @click="openLink(content)">
            <GlobeIcon />
          </Button>
          <Button v-tooltip="'Toot about it'" icon-only @click="sendToot" >
            <MastodonIcon />
          </Button>
          <Button v-tooltip="'Tweet about it'" icon-only @click="sendTweet">
            <TwitterIcon />
          </Button>
          <Button v-tooltip="'Share on Reddit'" icon-only @click="postOnReddit">
            <RedditIcon />
          </Button>
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
  gap: var(--gap-lg);
  padding: var(--gap-xl);
}

.all-buttons {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);

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
  }
}

.qr-wrapper {
  position: relative;

  &:hover {
    .copy-button {
      opacity: 1;
    }
  }
}

.qr-code {
  --size: 5rem;
  padding: var(--gap-sm);
  background-color: white !important;
}

.copy-button {
  position: absolute;
  top: 0;
  right: 0;
  margin: var(--gap-sm);
  transition: all 0.2s ease-in-out;
  opacity: 0;
}
</style>
