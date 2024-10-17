<template>
  <div class="flex h-full w-full flex-col gap-4">
    <h1 class="m-0 text-2xl font-bold leading-none text-contrast">Players</h1>

    <div class="grid grid-cols-3 gap-2 rounded-xl">
      <div
        v-for="player in players"
        :key="player.id"
        class="w-full items-center justify-between rounded-lg border border-solid border-divider p-2"
      >
        <div class="flex w-full items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <UiAvatar :src="player.avatar" size="sm" />
            <div class="flex flex-col gap-1">
              <h1 class="m-0 text-sm font-semibold leading-none text-primary">
                {{ player.name }}
              </h1>
              {{ player.created_at }}
            </div>
          </div>
          <ButtonStyled type="transparent">
            <OverflowMenu
              :options="[
                {
                  id: 'op',
                  action: () => {
                    emit('sendCommand', `op ${player.name}`);
                  },
                },
                {
                  id: 'deop',
                  action: () => {
                    emit('sendCommand', `deop ${player.name}`);
                  },
                },
                {
                  id: 'whitelist',
                  action: () => {
                    emit('sendCommand', `whitelist add ${player.name}`);
                  },
                },
                {
                  id: 'dewhitelist',
                  action: () => {
                    emit('sendCommand', `whitelist remove ${player.name}`);
                  },
                },
                {
                  id: 'kick',
                  action: () => {
                    emit('sendCommand', `kick ${player.name}`);
                  },
                },
                {
                  id: 'ban',
                  action: () => {
                    emit('sendCommand', `ban ${player.name}`);
                  },
                },
              ]"
              position="bottom"
            >
              <MoreHorizontalIcon class="h-5 w-5 bg-transparent text-contrast" />

              <template #op> Op </template>
              <template #deop> Deop </template>
              <template #whitelist> Whitelist </template>
              <template #dewhitelist> Dewhitelist </template>
              <template #kick> Kick </template>
              <template #ban> Ban </template>
            </OverflowMenu>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { MoreHorizontalIcon } from "@modrinth/assets";
import { OverflowMenu } from "@modrinth/ui";
import ButtonStyled from "@modrinth/ui/src/components/base/ButtonStyled.vue";

const emit = defineEmits(["sendCommand"]);

defineProps<{
  players: any[];
}>();
</script>
