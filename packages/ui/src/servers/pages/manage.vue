<template>
  <div class="flex flex-col sm:flex-row gap-4 sm:gap-0 my-4">
    <div class="flex flex-col gap-2">
      <span class="text-2xl sm:text-3xl text-contrast font-bold">Servers</span>
      <span class="text-sm sm:text-base text-secondary">View and manage all your servers</span>
    </div>
    <div class="sm:ml-auto">
      <ButtonStyled color="green" :size="isMobile ? 'standard' : 'large'" class="w-full sm:w-auto">
        <button class="flex items-center justify-center gap-2">
          <PlusIcon class="size-4" />
          <span>New server</span>
        </button>
      </ButtonStyled>
    </div>
  </div>

  <span class="text-md sm:text-lg text-contrast font-bold mb-4 flex flex-row gap-2">
    Your servers
  </span>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
    <ServerCard
      v-for="server in servers"
      :key="server.id"
      :server_name="server.server_name"
      :server_created="server.server_created"
      :server_plan="server.server_plan"
      :server_status="server.server_status"
      :players_online="server.players_online"
      :max_players_online="server.max_players_online"
      :world_name="server.world_name"
      :ip="server.ip"
      :id="server.id"
    />
  </div>

  <span class="text-md sm:text-lg text-contrast font-bold mb-4 flex flex-row gap-2">
    Servers shared with you
  </span>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
    <ServerCard
      v-for="server in servers"
      :key="server.id"
      :server_name="server.server_name"
      :server_created="server.server_created"
      :server_plan="server.server_plan"
      :server_status="server.server_status"
      :players_online="server.players_online"
      :max_players_online="server.max_players_online"
      :world_name="server.world_name"
      :ip="server.ip"
      :id="server.id"
    />
  </div>

  <div v-if="servers.length === 0" class="text-center py-12">
    <ServerIcon class="size-12 mx-auto text-gray-400 mb-4" />
    <h3 class="text-lg font-medium text-gray-900 mb-2">No servers found</h3>
    <p class="text-gray-500 mb-6 px-4">Get started by creating your first server</p>
    <ButtonStyled color="green" class="w-full sm:w-auto">
      <button class="flex items-center justify-center gap-2">
        <PlusIcon class="size-4" /> Create your first server
      </button>
    </ButtonStyled>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue'
import { ButtonStyled, RaisedBadge } from '@modrinth/ui'
import ServerCard from '../components/management/ServerCard.vue'
import { PlusIcon, ServerIcon } from '@modrinth/assets'

const servers = ref([
  {
    id: 'server-1',
    server_name: 'Rinth SMP',
    server_created: new Date('2023-10-01T12:00:00Z'),
    server_plan: 'Large',
    server_status: 'online',
    players_online: 5,
    max_players_online: 20,
    world_name: 'Example World',
    ip: 'valiant-apple.modrinth.gg',
  },
  {
    id: 'server-1',
    server_name: 'Rinth SMP',
    server_created: new Date('2023-10-01T12:00:00Z'),
    server_plan: 'Large',
    server_status: 'online',
    players_online: 5,
    max_players_online: 20,
    world_name: 'Example World',
    ip: 'valiant-apple.modrinth.gg',
  },
  {
    id: 'server-1',
    server_name: 'Rinth SMP',
    server_created: new Date('2023-10-01T12:00:00Z'),
    server_plan: 'Large',
    server_status: 'online',
    players_online: 5,
    max_players_online: 20,
    world_name: 'Example World',
    ip: 'valiant-apple.modrinth.gg',
  },
  {
    id: 'server-1',
    server_name: 'Rinth SMP',
    server_created: new Date('2023-10-01T12:00:00Z'),
    server_plan: 'Large',
    server_status: 'online',
    players_online: 5,
    max_players_online: 20,
    world_name: 'Example World',
    ip: 'valiant-apple.modrinth.gg',
  },
])
</script>
