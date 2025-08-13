<template>
  <Card>
    <div class="server-card-grid">
      <div class="header-section flex gap-4 items-center mb-4">
        <Avatar size="4rem" />
        <div class="flex flex-col gap-2">
          <span class="text-xl text-contrast font-bold">{{ server_name }}</span>
          <span class="text-md text-secondary" v-tooltip="server_created.toLocaleString()">
            Created {{ formatRelativeTime(server_created) }}
          </span>
        </div>
      </div>

      <div class="badges-section flex gap-2 items-center mb-4">
        <RaisedBadge>{{ server_plan }}</RaisedBadge>
        <RaisedBadge class="text-lg" :color="serverStatusColor">
          &bull; {{ formattedServerStatus }}
        </RaisedBadge>
      </div>

      <div class="content-section flex flex-col gap-2 mb-4">
        <div class="flex flex-row gap-2">
          <UsersIcon class="size-4 my-auto" />
          <span class="text-secondary">
            {{ players_online }} / {{ max_players_online }} players
          </span>
        </div>
        <div class="flex flex-row gap-2">
          <GlobeIcon class="size-4 my-auto" />
          <span class="text-secondary">{{ world_name }}</span>
        </div>
        <div class="flex flex-row gap-2">
          <LinkIcon class="size-4 my-auto" />
          <CopyCode :text="ip" />
        </div>
      </div>

      <div class="actions-section flex gap-2">
        <ButtonStyled color="brand">
          <RouterLink :to="`/servers/manage/${id}`">
            <EditIcon class="size-4" />
            Manage
          </RouterLink>
        </ButtonStyled>
        <ButtonStyled>
          <RouterLink :to="`/servers/manage/${id}`">
            <CurrencyIcon class="size-4" />
            Billing
          </RouterLink>
        </ButtonStyled>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { CurrencyIcon, EditIcon, GlobeIcon, LinkIcon, UsersIcon } from '@modrinth/assets'
import { Avatar, Card, RaisedBadge, useRelativeTime, CopyCode, ButtonStyled } from '@modrinth/ui'
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

const props = defineProps<{
  server_name: string
  server_created: Date
  server_plan: string
  server_status: string
  players_online: number
  max_players_online: number
  world_name: string
  ip: string
  id: string
}>()

const formatRelativeTime = useRelativeTime()

const serverStatusColor = computed(() => {
  switch (props.server_status) {
    case 'online':
      return 'green'
    case 'restarting':
      return 'orange'
    case 'offline':
      return undefined
    default:
      return undefined
  }
})

const formattedServerStatus = computed(() => {
  return props.server_status.slice(0, 1).toUpperCase() + props.server_status.slice(1)
})
</script>

<style scoped>
.server-card-grid {
  display: grid;
  grid-template-areas:
    'header badges'
    'content content'
    'actions actions';
  grid-template-columns: 1fr auto;
  align-items: start;
}

@media (max-width: 768px) {
  .server-card-grid {
    grid-template-areas:
      'header'
      'badges'
      'content'
      'actions';
    grid-template-columns: 1fr;
  }

  .badges-section {
    justify-self: start;
  }
}

@media (min-width: 769px) {
  .badges-section {
    justify-self: end;
  }
}

.header-section {
  grid-area: header;
}

.badges-section {
  grid-area: badges;
}

.content-section {
  grid-area: content;
}

.actions-section {
  grid-area: actions;
}
</style>
