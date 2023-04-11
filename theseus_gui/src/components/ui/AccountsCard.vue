<template>
  <Avatar
    class="button-base"
    ref="avatar"
    size="sm"
    :src="selectedAccount ? `https://crafthead.net/helm/${selectedAccount.id}` : ''"
    @click="toggle()"
  />
  <transition name="fade">
    <Card v-if="showCard" ref="card" class="account-card">
      <div v-if="selectedAccount" class="selected account">
        <Avatar size="xs" :src="`https://crafthead.net/helm/${selectedAccount.id}`" />
        <div>
          <h4>{{ selectedAccount.username }}</h4>
          <p>Selected</p>
        </div>
        <Button icon-only color="danger" @click="logout(selectedAccount.id)">
          <LogOutIcon />
        </Button>
      </div>
      <div v-else class="logged-out account">
        <h4>Not signed in</h4>
        <Button icon-only color="primary" @click="login()">
          <LoginIcon />
        </Button>
      </div>
      <div v-if="displayAccounts.length > 0" class="account-group">
        <div v-for="account in displayAccounts" :key="account.id" class="account-row">
          <Button class="option account" @click="setAccount(account)">
            <Avatar :src="account.profile_picture" class="icon" />
            <div>
              <p>{{ account.username }}</p>
            </div>
          </Button>
          <Button icon-only @click="logout(account.id)">
            <LogOutIcon />
          </Button>
        </div>
      </div>
      <Button v-if="accounts.length > 0" @click="login()">
        <PlusIcon />
        Add Account
      </Button>
    </Card>
  </transition>
</template>

<script setup>
import { Avatar, Button, Card, PlusIcon, LogOutIcon } from 'omorphia'
import { LoginIcon } from '@/assets/icons'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import {
  users,
  remove_user,
  authenticate_begin_flow,
  authenticate_await_completion,
} from '@/helpers/auth'
import { get, set } from '@/helpers/settings'
import { WebviewWindow } from '@tauri-apps/api/window'

const settings = ref(await get())

const appendProfiles = (accounts) => {
  return accounts.map((account) => {
    return {
      ...account,
      profile_picture: `https://crafthead.net/helm/${account.username}/128`,
    }
  })
}

const accounts = ref(await users().then(appendProfiles))

const displayAccounts = computed(() =>
  accounts.value.filter((account) => settings.value.default_user !== account.id)
)

const selectedAccount = ref(
  await users().then((accounts) =>
    accounts.find((account) => account.id === settings.value.default_user)
  )
)

const refreshValues = async () => {
  accounts.value = await users().then(appendProfiles)
  selectedAccount.value = accounts.value.find(
    (account) => account.id === settings.value.default_user
  )
}

let showCard = ref(false)
let card = ref(null)
let avatar = ref(null)

const setAccount = async (account) => {
  settings.value.default_user = account.id
  selectedAccount.value = account
  await set(settings.value)
}

const login = async () => {
  console.log('login process')
  const url = await authenticate_begin_flow()
  console.log(url)

  const window = new WebviewWindow('loginWindow', {
    url: url,
  })

  window.once('tauri://created', function () {
    console.log('webview created')
  })

  window.once('tauri://error', function (e) {
    console.log('webview error', e)
  })

  const loggedIn = await authenticate_await_completion()
  await setAccount(loggedIn)
  await refreshValues()
  await window.close()
}

const logout = async (id) => {
  await remove_user(id)
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
  if (
    card.value &&
    avatar.value.$el !== event.target &&
    card.value.$el !== event.target &&
    !card.value.$el.contains(event.target)
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
  top: 0;
  left: 5rem;
  z-index: 100;
  gap: 0.5rem;
  padding: 1rem;
  border: 1px solid var(--color-button-bg);
  width: max-content;

  &.hidden {
    display: none;
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
</style>
