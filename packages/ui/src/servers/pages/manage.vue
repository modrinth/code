<template>
  <div v-if="servers.length + sharedServers.length === 0" class="text-center py-24 relative">
    <ServersManageIllustration class="servers-manage-illustration" />
    <ServerIcon class="size-24 mx-auto text-contrast mb-4" />
    <h3 class="text-3xl font-medium text-contrast mb-2">No servers found</h3>
    <p class="text-gray-500 mb-6 px-4">Get started by creating your first server</p>
    <ButtonStyled color="green" size="large" type="outlined">
      <button class="flex items-center justify-center gap-2 mx-auto">
        <PlusIcon class="size-4" /> New server
      </button>
    </ButtonStyled>
  </div>

  <div v-else class="flex flex-col sm:flex-row gap-4 sm:gap-0 my-4">
    <div class="flex flex-col gap-2">
      <span class="text-3xl text-contrast font-bold">Servers</span>
      <span class="text-sm sm:text-base text-secondary">View and manage all your servers</span>
    </div>
    <div class="sm:ml-auto">
      <ButtonStyled color="green" size="large" class="w-full sm:w-auto">
        <button class="flex items-center justify-center gap-2">
          <PlusIcon class="size-4" />
          <span>New server</span>
        </button>
      </ButtonStyled>
    </div>
  </div>

  <template v-if="servers.length > 0">
    <span class="text-xl text-contrast font-bold mb-4 flex flex-row gap-2"> Your servers </span>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 gap-y-2">
      <ServerCard
        v-for="server in servers"
        :id="server.id"
        :key="server.id"
        :server_name="server.server_name"
        :server_created="server.server_created"
        :server_plan="server.server_plan"
        :server_status="server.server_status"
        :players_online="server.players_online"
        :max_players_online="server.max_players_online"
        :world_name="server.world_name"
        :ip="server.ip"
      />
    </div>
  </template>

  <template v-if="sharedServers.length > 0">
    <span class="text-xl text-contrast font-bold mb-4 flex flex-row gap-2"> Shared servers </span>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 gap-y-2">
      <ServerCard
        v-for="server in sharedServers"
        :id="server.id"
        :key="server.id"
        :server_name="server.server_name"
        :server_created="server.server_created"
        :server_plan="server.server_plan"
        :server_status="server.server_status"
        :players_online="server.players_online"
        :max_players_online="server.max_players_online"
        :world_name="server.world_name"
        :ip="server.ip"
      />
    </div>
  </template>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import { ButtonStyled } from '@modrinth/ui'
import ServerCard from '../components/management/ServerCard.vue'
import { PlusIcon, ServerIcon, ServersManageIllustration } from '@modrinth/assets'

const sharedServers = ref([
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

// const sharedServers = ref([])
// const servers = ref([])

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
])
</script>

<style scoped lang="scss">
.servers-manage-illustration {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: -1;
  width: 500px;
  height: 500px;
  opacity: 0.05;
  mask: linear-gradient(to bottom, rgba(0, 0, 0, 1) 0%, rgba(0, 0, 0, 0) 100%);
  -webkit-mask: linear-gradient(to bottom, rgba(0, 0, 0, 1) 0%, rgba(0, 0, 0, 0) 100%);
}
</style>
