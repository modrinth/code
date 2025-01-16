<script setup lang="ts">
import { RadioButtonChecked, RadioButtonIcon, PlusIcon, TrashIcon, LogInIcon } from '@modrinth/assets'
import { Avatar, Button, Accordion, ButtonStyled } from '@modrinth/ui'
import { SkinManagerIcon } from '@/assets/icons/index.js'
import { type Ref, type ComputedRef, ref, computed, onUnmounted } from 'vue'
import {
  users,
  remove_user,
  set_default_user,
  login as login_flow,
  get_default_user,
} from '@/helpers/auth'
import { handleError } from '@/store/state.js'
import { trackEvent } from '@/helpers/analytics'
import { process_listener } from '@/helpers/events'
import { handleSevereError } from '@/store/error.js'
import {
  cache_users_skins,
  cache_new_user_skin,
  account_heads as skinManagerAccountHeads,
  loaded_skins,
  get_heads,
  get_filters,
  selectedAccount as skinManagerAccount,
} from '@/helpers/skin_manager.js'
import { defineMessages, useVIntl } from '@vintl/vintl'
import type { MinecraftCredentials } from '@/helpers/types'

const { formatMessage } = useVIntl()

const emit = defineEmits(['change'])

const accounts: Ref<MinecraftCredentials[]> = ref([])
const defaultUser: Ref<string | undefined> = ref()

async function refreshValues() {
  defaultUser.value = await get_default_user().catch(handleError)
  accounts.value = await users().catch(handleError) ?? []

  accounts.value.sort((a, b) => a.username.localeCompare(b.username))

  await refreshSkins()
}

defineExpose({
  refreshValues,
})

const selectedAccount: ComputedRef<MinecraftCredentials | undefined> = computed(() => {
  const account = accounts.value.find((account) => account.id === defaultUser.value);
  (skinManagerAccount as Ref<MinecraftCredentials | undefined>).value = account
  return account
})

async function setAccount(account: MinecraftCredentials) {
  defaultUser.value = account.id
  await set_default_user(account.id).catch(handleError)
  emit('change')
}

async function login() {
  const loggedIn = await login_flow().catch(handleSevereError)

  if (loggedIn) {
    await cache_new_user_skin(loggedIn).catch(handleError)
    await get_heads()
    await setAccount(loggedIn)
    await refreshValues()
  }

  trackEvent('AccountLogIn')
}

async function logout(id: string) {
  await remove_user(id).catch(handleError)
  await refreshValues()
  if (!selectedAccount.value && accounts.value.length > 0) {
    await setAccount(accounts.value[0])
    await refreshValues()
  } else {
    emit('change')
  }
  trackEvent('AccountLogOut')
}

const unlisten = await process_listener(async (e: { event: string }) => {
  if (e.event === 'launched') {
    await refreshValues()
  }
})

async function refreshSkins() {
  await get_heads()
  loaded_skins.value = await cache_users_skins().catch(handleError)
  await get_heads()
  await get_filters()
}

const account_heads: Ref<Record<string, string>> = computed(() => skinManagerAccountHeads.value)

onUnmounted(() => {
  unlisten()
})

const messages = defineMessages({
  notSignedIn: {
    id: 'minecraft-account.not-signed-in',
    defaultMessage: 'Not signed in',
  },
  addAccount: {
    id: 'minecraft-account.add-account',
    defaultMessage: 'Add account',
  },
  removeAccount: {
    id: 'minecraft-account.remove-account',
    defaultMessage: 'Remove account',
  },
  changeSkin: {
    id: 'minecraft-account.change-skin',
    defaultMessage: 'Change skin',
  },
})

await refreshValues()
</script>
<template>
  <Accordion
    ref="button"
    class="w-full mt-2 bg-button-bg rounded-xl overflow-clip"
    button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
  >
    <template #title>
      <div class="flex gap-2">
        <Avatar
          size="36px"
          :src="
          selectedAccount
            ? account_heads[selectedAccount.id]
            : 'https://launcher-files.modrinth.com/assets/steve_head.png'
        "
        />
        <div class="flex flex-col items-start w-full">
          <span>{{ selectedAccount ? selectedAccount.username : 'Select account' }}</span>
          <span class="text-secondary text-xs">Minecraft account</span>
        </div>
      </div>
    </template>
    <div
      class="bg-button-bg pt-1 pb-2"
    >
      <div v-if="!selectedAccount" class="logged-out account">
        <h4>Not signed in</h4>
        <Button v-tooltip="'Log in'" icon-only color="primary" @click="login()">
          <LogInIcon />
        </Button>
      </div>
      <div v-if="accounts.length > 0" class="account-group">
        <div v-for="account in accounts" :key="account.id" class="flex gap-1 items-center">
          <button class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base" @click="setAccount(account)">
            <RadioButtonChecked v-if="selectedAccount && selectedAccount.id === account.id" class="w-5 h-5 text-brand" />
            <RadioButtonIcon v-else class="w-5 h-5 text-secondary" />
            <Avatar :src="account_heads[account.id]" size="24px" />
            <p class="m-0 truncate" :class="selectedAccount && selectedAccount.id === account.id ? `text-contrast font-semibold` : `text-primary`">{{ account.username }}</p>
          </button>
          <ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
            <button v-tooltip="formatMessage(messages.removeAccount)" class="mr-2" @click="logout(account.id)">
              <TrashIcon />
            </button>
          </ButtonStyled>
        </div>
      </div>
      <div class="flex flex-col gap-2 px-2 pt-2">
        <ButtonStyled v-if="accounts.length > 0" class="w-full">
          <button @click="login()">
            <PlusIcon />
            {{ formatMessage(messages.addAccount) }}
          </button>
        </ButtonStyled>
        <ButtonStyled v-if="accounts.length > 0">
          <router-link to="/skinmanager" class="w-full">
            <SkinManagerIcon />
            {{ formatMessage(messages.changeSkin) }}
          </router-link>
        </ButtonStyled>
      </div>
    </div>
  </Accordion>
</template>
