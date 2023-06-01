<template>
  <Modal ref="onboardingModal" header="Getting started">
    <div class="modal-body">
      <transition name="slide" mode="out-in">
        <div v-if="page === 1" key="1" class="banner-content swapped-page">
          <TextLogo class="text-logo" />
          <h2>Getting started</h2>
          <p>
            The Modrinth App is a desktop application that allows you to easily install and manage
            Minecraft modpacks.
          </p>
          <Button color="primary" @click="page = 2"> Continue </Button>
        </div>
        <div v-else-if="page === 2" key="2" class="content swapped-page">
          <h2>Sign into Minecraft</h2>
          <AccountsCard mode="isolated" />
        </div>
        <div v-else-if="page === 3" key="3" class="content swapped-page">
          <h2>Setting up Java</h2>
          <div class="settings-group">
            <h3>Java 17 location</h3>
            <JavaSelector v-model="settings.java_globals.JAVA_17" :version="17" compact />
          </div>
          <div class="settings-group">
            <h3>Java 8 location</h3>
            <JavaSelector v-model="settings.java_globals.JAVA_8" :version="8" compact />
          </div>
        </div>
      </transition>
      <div class="button-row">
        <Button v-if="page !== 1" @click="page--"> Back </Button>
        <div class="page-indicator">
          <span class="circle" :class="{ active: page === 1 }" />
          <span class="circle" :class="{ active: page === 2 }" />
          <span class="circle" :class="{ active: page === 3 }" />
        </div>
        <Button v-if="page !== 1" :color="page === 3 ? 'primary' : ''" @click="pageTurn()">
          {{ page === 3 ? 'Finish' : 'Next' }}
        </Button>
      </div>
    </div>
  </Modal>
</template>

<script setup>
import { Modal, TextLogo, Button } from 'omorphia'
import { ref, watch } from 'vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import { handleError } from '@/store/notifications.js'
import { get, set } from '@/helpers/settings.js'

const onboardingModal = ref(null)
const page = ref(1)

defineExpose({
  show: () => {
    onboardingModal.value.show()
  },
})

const fetchSettings = await get().catch(handleError)

if (!fetchSettings.java_globals.JAVA_8)
  fetchSettings.java_globals.JAVA_8 = { path: '', version: '' }
if (!fetchSettings.java_globals.JAVA_17)
  fetchSettings.java_globals.JAVA_17 = { path: '', version: '' }

const settings = ref(fetchSettings)

watch(settings.value, async (oldSettings, newSettings) => {
  const setSettings = JSON.parse(JSON.stringify(newSettings))

  if (setSettings.java_globals.JAVA_8?.path === '') {
    setSettings.java_globals.JAVA_8 = undefined
  }
  if (setSettings.java_globals.JAVA_17?.path === '') {
    setSettings.java_globals.JAVA_17 = undefined
  }

  await set(setSettings)
})

const pageTurn = () => {
  if (page.value === 3) {
    onboardingModal.value.hide()
    return
  }
  page.value++
}
</script>

<style scoped lang="scss">
.modal-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: var(--gap-lg);

  h2 {
    margin-bottom: 0;
    margin-top: 0;
  }
}

.swapped-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.settings-group {
  width: 100%;
  text-align: left;
}

.button-row {
  display: flex;
  justify-content: space-between;
  flex-direction: row;
  align-items: center;
  width: 100%;

  .page-indicator {
    flex-grow: 1;
    margin: 0 auto;
  }
}

.text-logo {
  width: 100%;
  height: 100%;
}

.banner-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 4rem;
}

.content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  width: 100%;
}

.page-enter-active,
.page-leave-active {
  transition: opacity 0.3s ease;
}

.page-enter-from,
.page-leave-to {
  opacity: 0;
}

.circle {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  display: inline-block;
  margin-right: 0.25rem;
  background-color: var(--color-base);
  transition: all 0.3s ease-in-out;

  &.active {
    background-color: var(--color-brand);
  }
}

.table {
  .table-row {
    grid-template-columns: 1fr 4fr 1.5fr;
  }

  span {
    display: inherit;
    align-items: center;
    justify-content: center;
  }
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.5s;
}

.slide-enter,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
