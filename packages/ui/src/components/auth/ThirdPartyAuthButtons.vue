<template>
  <section class="grid grid-cols-[repeat(auto-fit,minmax(10rem,1fr))] gap-3">
    <template v-for="service in services" :key="service.id">
      <ButtonStyled>
        <a v-if="type === 'href'" :href="handler(service.id) as string">
          <component :is="service.icon" />
          <span>{{ service.formattedName }}</span>
        </a>
        <button v-else @click="handler(service.id)" class="w-full">
          <component :is="service.icon" />
          <span>{{ service.formattedName }}</span>
        </button>
      </ButtonStyled>
    </template>
  </section>
</template>

<script setup lang="ts">
import { type Component, type Ref, ref } from 'vue'

import {
  SSODiscordIcon,
  SSOGitHubIcon,
  SSOGitLabIcon,
  SSOGoogleIcon,
  SSOMicrosoftIcon,
  SSOSteamIcon,
} from '@modrinth/assets'
import ButtonStyled from '../base/ButtonStyled.vue'

type Service = {
  id: string
  icon: Component
  formattedName: string
}

const services: Ref<Service[]> = ref([
  {
    id: 'discord',
    icon: SSODiscordIcon,
    formattedName: 'Discord',
  },
  {
    id: 'github',
    icon: SSOGitHubIcon,
    formattedName: 'GitHub',
  },
  {
    id: 'microsoft',
    icon: SSOMicrosoftIcon,
    formattedName: 'Microsoft',
  },
  {
    id: 'google',
    icon: SSOGoogleIcon,
    formattedName: 'Google',
  },
  {
    id: 'steam',
    icon: SSOSteamIcon,
    formattedName: 'Steam',
  },
  {
    id: 'gitlab',
    icon: SSOGitLabIcon,
    formattedName: 'GitLab',
  },
])

type ServiceId = (typeof services.value)[number]['id']

export type HrefProvider = (service: ServiceId) => string
export type ActionProvider = (service: ServiceId) => void

defineProps<
  | {
      type: 'href'
      handler: HrefProvider
    }
  | {
      type: 'action'
      handler: ActionProvider
    }
>()
</script>
