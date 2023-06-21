<template>
  <div
    v-if="mode !== 'isolated'"
    ref="button"
    class="button-base avatar-button"
    :class="{ expanded: mode === 'expanded' }"
    @click="showCard = !showCard"
  >
    <Avatar
      :size="mode === 'expanded' ? 'xs' : 'sm'"
      :src="selectedAccount ? `https://mc-heads.net/avatar/${selectedAccount.id}/128` : ''"
    />
    <div v-show="mode === 'expanded'" class="avatar-text">
      <div class="text no-select">
        {{ selectedAccount ? selectedAccount.username : 'Offline' }}
      </div>
      <p class="accounts-text no-select">
        <UsersIcon />
        Accounts
      </p>
    </div>
  </div>
  <transition name="fade">
    <Card
      v-if="showCard || mode === 'isolated'"
      ref="card"
      class="account-card"
      :class="{ expanded: mode === 'expanded', isolated: mode === 'isolated' }"
    >
      <div v-if="selectedAccount" class="selected account">
        <Avatar size="xs" :src="`https://mc-heads.net/avatar/${selectedAccount.id}/128`" />
        <div>
          <h4>{{ selectedAccount.username }}</h4>
          <p>Selected</p>
        </div>
        <Button v-tooltip="'Log out'" icon-only color="raised" @click="logout(selectedAccount.id)">
          <TrashIcon />
        </Button>
      </div>
      <div v-else class="logged-out account">
        <h4>Not signed in</h4>
        <Button v-tooltip="'Log in'" icon-only color="primary" @click="login()">
          <LogInIcon />
        </Button>
      </div>
      <div v-if="displayAccounts.length > 0" class="account-group">
        <div v-for="account in displayAccounts" :key="account.id" class="account-row">
          <Button class="option account" @click="setAccount(account)">
            <Avatar :src="`https://mc-heads.net/avatar/${account.id}/128`" class="icon" />
            <p>{{ account.username }}</p>
          </Button>
          <Button v-tooltip="'Log out'" icon-only @click="logout(account.id)">
            <TrashIcon />
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
import { Avatar, Button, Card, PlusIcon, TrashIcon, UsersIcon, LogInIcon } from 'omorphia'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import {
  users,
  remove_user,
  authenticate_begin_flow,
  authenticate_await_completion,
} from '@/helpers/auth'
import { get, set } from '@/helpers/settings'
import { WebviewWindow } from '@tauri-apps/api/window'
import { handleError } from '@/store/state.js'
import mixpanel from 'mixpanel-browser'

defineProps({
  mode: {
    type: String,
    required: true,
    default: 'normal',
  },
})

const emit = defineEmits(['change'])

const settings = ref({})
const accounts = ref([])
async function refreshValues() {
  settings.value = await get().catch(handleError)
  accounts.value = await users().catch(handleError)
}
defineExpose({
  refreshValues,
})
await refreshValues()

const displayAccounts = computed(() =>
  accounts.value.filter((account) => settings.value.default_user !== account.id)
)

const selectedAccount = computed(() =>
  accounts.value.find((account) => account.id === settings.value.default_user)
)

async function setAccount(account) {
  settings.value.default_user = account.id
  await set(settings.value).catch(handleError)
  emit('change')
}

async function login() {
  const url = await authenticate_begin_flow().catch(handleError)

  const window = new WebviewWindow('loginWindow', {
    title: 'Modrinth App',
    url: url,
  })

  const loggedIn = await authenticate_await_completion().catch(handleError)
  await setAccount(loggedIn)
  await refreshValues()
  await window.close()
  mixpanel.track('AccountLogIn')
}

const logout = async (id) => {
  await remove_user(id).catch(handleError)
  await refreshValues()
  if (!selectedAccount.value && accounts.value.length > 0) {
    await setAccount(accounts.value[0])
    await refreshValues()
  } else {
    emit('change')
  }
  mixpanel.track('AccountLogOut')
}

let showCard = ref(false)
let card = ref(null)
let button = ref(null)
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

  h4,
  p {
    margin: 0;
  }
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

  &.isolated {
    position: relative;
    left: 0;
    top: 0;
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
  background-color: var(--color-raised-bg);
  border-radius: var(--radius-md);
  width: 100%;
  text-align: left;

  &.expanded {
    border: 1px solid var(--color-button-bg);
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

.accounts-text {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin: 0;
}
</style>
