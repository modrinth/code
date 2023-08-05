<script setup>
import { Button, LogInIcon, Modal, ClipboardCopyIcon, GlobeIcon, Card } from 'omorphia'
import { authenticate_await_completion, authenticate_begin_flow } from '@/helpers/auth.js'
import { handleError } from '@/store/notifications.js'
import mixpanel from 'mixpanel-browser'
import { get, set } from '@/helpers/settings.js'
import { ref } from 'vue'
import QrcodeVue from 'qrcode.vue'

const loginUrl = ref(null)
const loginModal = ref()

const props = defineProps({
  nextPage: {
    type: Function,
    required: true,
  },
  prevPage: {
    type: Function,
    required: true,
  },
})

async function login() {
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
  loginModal.value.hide()

  props.nextPage(loggedIn[1])
  const settings = await get().catch(handleError)
  settings.default_user = loggedIn[0].id
  await set(settings).catch(handleError)
  await mixpanel.track('AccountLogIn')
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
</script>

<template>
  <div class="login-card">
    <img
      src="https://launcher-files.modrinth.com/assets/default_profile.png"
      class="logo"
      alt="Minecraft art"
    />
    <Card class="logging-in">
      <h2>Sign into Minecraft</h2>
      <p>
        Sign in with your Microsoft account to launch Minecraft with your mods and modpacks. If you
        don't have a Minecraft account, you can purchase the game on the
        <a
          href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc"
          class="link"
        >
          Minecraft website
        </a>
      </p>
      <div class="action-row">
        <Button class="transparent" large @click="prevPage"> Back </Button>
        <div class="sign-in-pair">
          <Button color="primary" large @click="login">
            <LogInIcon v-if="!finalizedLogin" />
            {{ finalizedLogin ? 'Next' : 'Sign in' }}
          </Button>
          <Button v-if="loginUrl" class="transparent" @click="loginModal.show()">
            Browser didn't open?
          </Button>
        </div>
        <Button class="transparent" large @click="nextPage()"> Next </Button>
      </div>
    </Card>
  </div>
  <Modal ref="loginModal" header="Signing in">
    <div class="modal-body">
      <QrcodeVue :value="loginUrl" class="qr-code" margin="3" size="160" />
      <div class="modal-text">
        <p>
          Sign into Microsoft with your browser. If your browser didn't open, you can copy and open
          the link below, or scan the QR code with your device.
        </p>
        <div class="iconified-input">
          <LogInIcon />
          <input type="text" :value="loginUrl" readonly />
          <Button
            v-tooltip="'Copy link'"
            icon-only
            color="raised"
            @click="() => navigator.clipboard.writeText(loginUrl)"
          >
            <ClipboardCopyIcon />
          </Button>
        </div>
        <div class="button-row">
          <Button @click="openUrl">
            <GlobeIcon />
            Open link
          </Button>
          <Button class="transparent" @click="loginModal.hide"> Cancel </Button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.login-card {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  margin: auto;
  padding: var(--gap-lg);
  width: 30rem;

  img {
    width: 100%;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  }
}

.logging-in {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  vertical-align: center;
  gap: var(--gap-md);
  background-color: var(--color-raised-bg);
  width: 100%;
  border-radius: 0 0 var(--radius-lg) var(--radius-lg);

  h2,
  p {
    margin: 0;
  }

  p {
    text-align: center;
  }
}

.link {
  color: var(--color-blue);
  text-decoration: underline;
}

.button-row {
  display: flex;
  flex-direction: row;
}

.action-row {
  width: 100%;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-md);
  margin-top: var(--gap-md);

  .transparent {
    padding: 0 var(--gap-md);
  }
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

    h2,
    p {
      margin: 0;
    }
  }
}

.sticker {
  width: 100%;
  max-width: 25rem;
  height: auto;
  margin-bottom: var(--gap-lg);
}

.sign-in-pair {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  align-items: center;
}
</style>
