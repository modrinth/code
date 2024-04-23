<script setup>
import { Modal, XIcon, IssuesIcon, LogInIcon } from 'omorphia'
import { ChatIcon } from '@/assets/icons'
import { ref } from 'vue'
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { handleError } from '@/store/notifications.js'
import mixpanel from 'mixpanel-browser'
import { handleSevereError } from '@/store/error.js'

const errorModal = ref()
const error = ref()

const title = ref('An error occurred')
const errorType = ref('unknown')
const supportLink = ref('https://support.modrinth.com')
const metadata = ref({})

defineExpose({
  async show(errorVal) {
    if (errorVal.message && errorVal.message.includes('Minecraft authentication error:')) {
      title.value = 'Unable to sign in to Minecraft'
      errorType.value = 'minecraft_auth'
      supportLink.value =
        'https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues'

      if (errorVal.message.includes('existing connection was forcibly closed')) {
        metadata.value.network = true
      }
      if (errorVal.message.includes('because the target machine actively refused it')) {
        metadata.value.hostsFile = true
      }
    } else if (errorVal.message && errorVal.message.includes('User is not logged in')) {
      title.value = 'Sign in to Minecraft'
      errorType.value = 'minecraft_sign_in'
      supportLink.value = 'https://support.modrinth.com'
    } else {
      title.value = 'An error occurred'
      errorType.value = 'unknown'
      supportLink.value = 'https://support.modrinth.com'
      metadata.value = {}
    }

    error.value = errorVal
    errorModal.value.show()
  },
})

const loadingMinecraft = ref(false)
async function loginMinecraft() {
  try {
    loadingMinecraft.value = true
    const loggedIn = await login_flow()

    if (loggedIn) {
      await set_default_user(loggedIn.id).catch(handleError)
    }

    await mixpanel.track('AccountLogIn')
    loadingMinecraft.value = false
    errorModal.value.hide()
  } catch (err) {
    loadingMinecraft.value = false
    handleSevereError(err)
  }
}
</script>

<template>
  <Modal ref="errorModal" :header="title">
    <div class="modal-body">
      <div class="markdown-body">
        <template v-if="errorType === 'minecraft_auth'">
          <p>
            Signing into Microsoft account is a complex task for the launchers, and there are a lot
            of things can go wrong.
          </p>
          <template v-if="metadata.network">
            <h3>Network issues</h3>
            <p>
              It looks like there were issues with the Modrinth App connecting to Microsoft's
              servers. This is often the result of a poor connection, so we recommend trying again
              to see if it works. If issues continue to persist, follow the steps in
              <a
                href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_e71a5f805f"
              >
                our support article
              </a>
              to troubleshoot.
            </p>
          </template>
          <template v-else-if="metadata.hostsFile">
            <h3>Network issues</h3>
            <p>
              The Modrinth App tried to connect to Microsoft / Xbox / Minecraft services, but the
              remote server rejected the connection. This may indicate that these services are
              blocked by the hosts file. Please visit
              <a
                href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_d694a29256"
              >
                our support article
              </a>
              for steps on how to fix the issue.
            </p>
          </template>
          <template v-else>
            <h3>Make sure you are signing into the right Microsoft account</h3>
            <p>
              More often than not, this error is caused by you signing into an incorrect Microsoft
              account which isn't linked to Minecraft. Double check and try again!
            </p>
            <h3>Try signing in and launching through the official launcher first</h3>
            <p>
              If you just bought Minecraft, are coming from the Bedrock Edition world and have never
              played Java before, or just subscribed to PC Game Pass, you would need to start the
              game at least once using the
              <a href="https://www.minecraft.net/en-us/download">official Minecraft Launcher</a>.
              Once you're done, come back here and sign in!
            </p>
          </template>
          <div class="cta-button">
            <button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
              <LogInIcon /> Sign in to Minecraft
            </button>
          </div>
          <hr />
          <p>
            If nothing is working and you need help, visit
            <a :href="supportLink">our support page</a>
            and start a chat using the widget in the bottom right and we will be more than happy to
            assist!
          </p>
          <details>
            <summary>Debug info</summary>
            {{ error.message ?? error }}
          </details>
        </template>
        <div v-else-if="errorType === 'minecraft_sign_in'">
          <div class="warning-banner">
            <div class="warning-banner__title">
              <IssuesIcon />
              <span>Installed the app before April 23rd, 2024?</span>
            </div>
            <div class="warning-banner__description">
              Modrinth has updated our sign-in workflow to allow for better stability, security, and
              performance. You must sign in again so your credentials can be upgraded to this new
              flow.
            </div>
          </div>
          <p>
            To play this instance, you must sign in through Microsoft below. If you don't have a
            Minecraft account, you can purchase the game on the
            <a href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc"
              >Minecraft website</a
            >.
          </p>
          <div class="cta-button">
            <button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
              <LogInIcon /> Sign in to Minecraft
            </button>
          </div>
        </div>
        <template v-else>
          {{ error.message ?? error }}
        </template>
      </div>
      <div class="input-group push-right">
        <a :href="supportLink" class="btn" @click="errorModal.hide()"><ChatIcon /> Get support</a>
        <button class="btn" @click="errorModal.hide()"><XIcon /> Close</button>
      </div>
    </div>
  </Modal>
</template>

<style>
.light-mode {
  --color-orange-bg: rgba(255, 163, 71, 0.2);
}

.dark-mode,
.oled-mode {
  --color-orange-bg: rgba(224, 131, 37, 0.2);
}
</style>

<style scoped lang="scss">
.cta-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem;
}

.warning-banner {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: var(--gap-lg);
  background-color: var(--color-orange-bg);
  border: 2px solid var(--color-orange);
  border-radius: var(--radius-md);
  margin-bottom: 1rem;
}

.warning-banner__title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 700;

  svg {
    color: var(--color-orange);
    height: 1.5rem;
    width: 1.5rem;
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
  padding: var(--gap-lg);
}
</style>
