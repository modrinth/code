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
      const dateACreated = dayjs(a.created)
      const dateAPlayed = a.last_played ? dayjs(a.last_played) : dayjs(0)

      const dateBCreated = dayjs(b.created)
      const dateBPlayed = b.last_played ? dayjs(b.last_played) : dayjs(0)

      const dateA = dateACreated.isAfter(dateAPlayed) ? dateACreated : dateAPlayed
      const dateB = dateBCreated.isAfter(dateBPlayed) ? dateBCreated : dateBPlayed

      if (dateA.isSame(dateB)) {
        return a.name.localeCompare(b.name)
      }

      return dateB - dateA
    })
    .slice(0, 4)
}

await getInstances()

const unlistenProfile = await profile_listener(async (event) => {
  if (event.event !== 'synced') {
    await getInstances()
  }
})

onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <NavButton
    v-for="instance in recentInstances"
    :key="instance.id"
    v-tooltip.right="instance.name"
    :to="`/instance/${encodeURIComponent(instance.path)}`"
    class="relative"
  >
    <Avatar
      :src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
      size="28px"
      :tint-by="instance.path"
      :class="`transition-all ${instance.install_stage !== 'installed' ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
    />
    <div
      v-if="instance.install_stage !== 'installed'"
      class="absolute inset-0 flex items-center justify-center z-10"
    >
      <SpinnerIcon class="animate-spin w-4 h-4" />
    </div>
  </NavButton>
  <div v-if="recentInstances.length > 0" class="h-px w-6 mx-auto my-2 bg-button-bg"></div>
</template>

<style scoped lang="scss"></style>
