<script setup>
import { Modal, XIcon } from 'omorphia'
import { ChatIcon } from '@/assets/icons'
import { ref } from 'vue'

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

          <hr />
          <p>
            If nothing is working and you need help, visit
            <a :href="supportLink">our support page</a>
            and start a chat using the widget in the bottom right and we will be more than happy to
            assist!
          </p>
          <p>Debug info: {{ error.message ?? error }}</p>
        </template>
        <template v-else>
          {{ error.message ?? error }}
        </template>
      </div>
      <div class="input-group push-right">
        <a :href="supportLink" class="btn" @click="errorModal.hide()"><ChatIcon /> Get support</a>
        <button class="btn btn-primary" @click="errorModal.hide()"><XIcon /> Close</button>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.modal-body {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
  padding: var(--gap-lg);
}
</style>
