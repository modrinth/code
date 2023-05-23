<template>
  <div
    ref="button"
    class="button-base avatar-button"
    :class="{ expanded: expanded }"
    @click="toggle()"
  >
    <Avatar :size="expanded ? 'xs' : 'sm'" :src="selectedAccount?.profile_picture ?? ''" />
    <div v-show="expanded" class="avatar-text">
      <div class="text no-select">
        {{ selectedAccount ? selectedAccount.username : 'Offline' }}
      </div>
      <p class="no-select">
        <UsersIcon />
        Accounts
      </p>
    </div>
  </div>
  <transition name="fade">
    <Card v-if="showCard" ref="card" class="account-card" :class="{ expanded: expanded }">
      <div v-if="selectedAccount" class="selected account">
        <Avatar size="xs" :src="selectedAccount.profile_picture" />
        <div>
          <h4>{{ selectedAccount.username }}</h4>
          <p>Selected</p>
        </div>
        <Button icon-only color="raised" @click="logout(selectedAccount.id)">
          <XIcon />
        </Button>
      </div>
      <div v-else class="logged-out account">
        <h4>Not signed in</h4>
        <Button icon-only color="primary" @click="login()">
          <LogInIcon />
        </Button>
      </div>
      <div v-if="displayAccounts.length > 0" class="account-group">
        <div v-for="account in displayAccounts" :key="account.id" class="account-row">
          <Button class="option account" @click="setAccount(account)">
            <Avatar :src="account.profile_picture" class="icon" />
            <p>{{ account.username }}</p>
          </Button>
          <Button icon-only @click="logout(account.id)">
            <XIcon />
          </Button>
        </div>
      </div>
      <Button v-if="accounts.length > 0" @click="login()">
        <PlusIcon />
        Add account
      </Button>
    </Card>
  </transition>
</template>

<script setup>
import { Avatar, Button, Card, PlusIcon, XIcon, UsersIcon, LogInIcon } from 'omorphia'
import { ref, defineProps, computed, onMounted, onBeforeUnmount } from 'vue'
import {
  users,
  remove_user,
  authenticate_begin_flow,
  authenticate_await_completion,
} from '@/helpers/auth'
import { get, set } from '@/helpers/settings'
import { WebviewWindow } from '@tauri-apps/api/window'
import { handleError } from '@/store/state.js'

defineProps({
  expanded: {
    type: Boolean,
    required: true,
  },
})

const settings = ref(await get().catch(handleError))

const appendProfiles = (accounts) => {
  return accounts.map((account) => {
    return {
      ...account,
      profile_picture: `https://crafthead.net/helm/${account.id.replace(/-/g, '')}/128`,
    }
  })
}

const accounts = ref(await users().then(appendProfiles).catch(handleError))

const displayAccounts = computed(() =>
  accounts.value.filter((account) => settings.value.default_user !== account.id)
)

const selectedAccount = ref(
  accounts.value.find((account) => account.id === settings.value.default_user)
)

const refreshValues = async () => {
  accounts.value = await users().then(appendProfiles).catch(handleError)
  selectedAccount.value = accounts.value.find(
    (account) => account.id === settings.value.default_user
  )
}

let showCard = ref(false)
let card = ref(null)
let button = ref(null)

const setAccount = async (account) => {
  settings.value.default_user = account.id
  selectedAccount.value = account
  await set(settings.value).catch(handleError)
}

const login = async () => {
  const url = await authenticate_begin_flow().catch(handleError)

  const window = new WebviewWindow('loginWindow', {
    title: 'Modrinth App',
    url: url,
  })

  window.once('tauri://created', function () {
    console.log('webview created')
  })

  window.once('tauri://error', function (e) {
    console.log('webview error', e)
  })

  const loggedIn = await authenticate_await_completion().catch(handleError)
  await setAccount(loggedIn)
  await refreshValues()
  await window.close()
}

const logout = async (id) => {
  await remove_user(id).catch(handleError)
  await refreshValues()
  if (!selectedAccount.value && accounts.value.length > 0) {
    await setAccount(accounts.value[0])
    await refreshValues()
  }
}

const toggle = () => {
  showCard.value = !showCard.value
}

const handleClickOutside = (event) => {
  const elements = document.elementsFromPoint(event.clientX, event.clientY)
  if (
    card.value &&
    card.value.$el !== event.target &&
    !elements.includes(card.value.$el) &&
    !button.value.contains(event.target)
  ) {
    showCard.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped lang="scss">
.selected {
  background: var(--color-brand-highlight);
  border-radius: var(--radius-lg);
  color: var(--color-contrast);
  gap: 1rem;
}

.logged-out {
  background: var(--color-bg);
  border-radius: var(--radius-lg);
  gap: 1rem;
}

.account {
  width: max-content;
  display: flex;
  align-items: center;
  text-align: left;
  padding: 0.5rem 1rem;
}

.account-card {
  position: absolute;
  display: flex;
  flex-direction: column;
  top: 0.5rem;
  left: 5.5rem;
  z-index: 9;
  gap: 0.5rem;
  padding: 1rem;
  border: 1px solid var(--color-button-bg);
  width: max-content;
  user-select: none;
  -ms-user-select: none;
  -webkit-user-select: none;

  &.hidden {
    display: none;
  }

  &.expanded {
    left: 13.5rem;
  }
}

.accounts-title {
  font-size: 1.2rem;
  font-weight: bolder;
}

.account-group {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.option {
  width: calc(100% - 2.25rem);
  background: var(--color-raised-bg);
  color: var(--color-base);
  box-shadow: none;

  img {
    margin-right: 0.5rem;
  }
}

.icon {
  --size: 1.5rem !important;
}

.account-row {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  vertical-align: center;
  justify-content: space-between;
  padding-right: 1rem;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.avatar-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-base);
  background-color: var(--color-bg);
  border-radius: var(--radius-md);
  box-shadow: none;
  width: 100%;
  text-align: left;

  &.expanded {
    padding: 1rem;
  }
}

.avatar-text {
  margin: auto 0 auto 0.25rem;
  display: flex;
  flex-direction: column;
}

.text {
  width: 6rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
