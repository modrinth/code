<script setup>
import {
  Button,
  LogInIcon,
  Modal,
  ClipboardCopyIcon,
  GlobeIcon
} from "omorphia";
import {authenticate_await_completion, authenticate_begin_flow} from "@/helpers/auth.js";
import {handleError} from "@/store/notifications.js";
import mixpanel from "mixpanel-browser";
import {get, set} from "@/helpers/settings.js";
import {ref} from "vue";
import QrcodeVue from 'qrcode.vue'
import {LoginSticker} from "@/assets/images";

const loginUrl = ref(null)
const loginModal = ref()
const finalizedLogin = ref(false)

async function login() {
  if (!finalizedLogin.value) {
    const url = await authenticate_begin_flow().catch(handleError)
    loginUrl.value = url

    await window.__TAURI_INVOKE__('tauri', {
      __tauriModule: 'Shell',
      message: {
        cmd: 'open',
        path: url,
      },
    })

    const loggedIn = await authenticate_await_completion().catch(handleError)
    finalizedLogin.value = true
    loginModal.value.hide()
    const settings = await get().catch(handleError)
    settings.default_user = loggedIn.id
    await set(settings.value).catch(handleError)
    mixpanel.track('AccountLogIn')
  } else {
    props.nextPage();
  }
}

const openUrl = async () => {
  await window.__TAURI_INVOKE__('tauri', {
    __tauriModule: 'Shell',
    message: {
      cmd: 'open',
      path: loginUrl.value,
    },
  })
}

const props = defineProps({
  nextPage: {
    type: Function,
    required: true,
  },
})
</script>

<template>
  <div class="logging-in">
    <LoginSticker class="sticker"/>
    <h1>Sign into Minecraft</h1>
    <p>Sign in with your Minecraft account to play with installed mods and modpacks.</p>
    <div class="button-row">
      <Button color="primary" large @click="login">
        <LogInIcon v-if="!finalizedLogin"/>
        {{ finalizedLogin ? 'Next' : 'Sign in' }}
      </Button>
      <Button v-if="loginUrl && !finalizedLogin" class="transparent" large @click="loginModal.show()">
        Browser didn't open?
      </Button>
    </div>
  </div>
  <Modal ref="loginModal" header="Signing in">
    <div class="modal-body">
      <QrcodeVue :value="loginUrl" class="qr-code" margin="3" size="160"/>
      <div class="modal-text">
        <p>
          Sign into Microsoft with your browser.
          If your browser didn't open, you can copy and open the link below,
          or scan the QR code with your device.
        </p>
        <div class="iconified-input">
          <LogInIcon/>
          <input type="text" :value="loginUrl" readonly/>
          <Button
            icon-only
            color="raised"
            v-tooltip="'Copy link'"
            @click="() => navigator.clipboard.writeText(loginUrl)"
          >
            <ClipboardCopyIcon/>
          </Button>
        </div>
        <div class="button-row">
          <Button @click="openUrl">
            <GlobeIcon/>
            Open link
          </Button>
          <Button class="transparent" @click="loginModal.hide">
            Cancel
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.logging-in {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  width: 100%;
  vertical-align: center;
  margin: auto;
  gap: var(--gap-xl);
  padding: calc(var(--gap-xl) * 2);
  border-radius: var(--radius-lg);
  background-color: var(--color-raised-bg);

  h1, p {
    margin: 0;
  }

  p {
    font-size: var(--font-size-lg);
    width: 30rem;
    text-align: center;
  }
}

.button-row {
  display: flex;
  flex-direction: row;
}

.qr-code {
  background-color: white !important;
  border-radius: var(--radius-md);
}

.modal-body {
  display: flex;
  flex-direction: row;
  gap: var(--gap-lg);
  align-items: center;
  padding: var(--gap-lg);

  .modal-text {
    display: flex;
    flex-direction: column;
    gap: var(--gap-sm);

    h2, p {
      margin: 0
    }
  }
}

.sticker {
  width: 100%;
  max-width: 25rem;
  height: auto;
  margin-bottom: var(--gap-lg);
}
</style>
