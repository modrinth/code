<script setup>
import { list } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import dayjs from 'dayjs'
import { onUnmounted, ref } from 'vue'
import { profile_listener } from '@/helpers/events.js'
import NavButton from '@/components/ui/NavButton.vue'
import { Avatar } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { SpinnerIcon } from '@modrinth/assets'

const recentInstances = ref([])
const getInstances = async () => {
  const profiles = await list().catch(handleError)

  recentInstances.value = profiles
    .sort((a, b) => {
      const dateA = dayjs(a.created > a.last_played ? a.last_played : a.created)
      const dateB = dayjs(b.created > b.last_played ? b.last_played : b.created)

      if (dateA.isSame(dateB)) {
        return a.name.localeCompare(b.name)
      }

      return dateB - dateA
    })
    .slice(0, 4)
}

await getInstances()

const unlistenProfile = await profile_listener(async () => {
  await getInstances()
})

onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <NavButton
    v-for="instance in recentInstances"
    :key="instance.id"
    :to="`/instance/${encodeURIComponent(instance.path)}`"
  >
    <Avatar
      :src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
      circle
      :class="`transition-all ${instance.install_stage !== 'installed' ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
    />
    <div
      v-if="instance.install_stage !== 'installed'"
      class="absolute inset-0 flex items-center justify-center"
    >
      <SpinnerIcon class="animate-spin w-4 h-4" />
    </div>
    <template #label>{{ instance.name }}</template>
  </NavButton>
</template>

<style scoped lang="scss"></style>
