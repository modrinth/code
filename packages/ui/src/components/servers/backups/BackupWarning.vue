<script setup lang="ts">
import { IssuesIcon } from '@modrinth/assets'
import AutoLink from '../../base/AutoLink.vue'

defineProps<{
  backupLink: string
}>()
</script>
<template>
  <div
    class="flex gap-3 rounded-2xl border-2 border-solid border-orange bg-bg-orange px-4 py-3 font-medium text-contrast"
  >
    <IssuesIcon class="mt-1 h-5 w-5 shrink-0 text-orange" />
    <span class="leading-normal">
      You may want to
      <AutoLink
        :to="backupLink"
        class="font-semibold text-orange hover:underline active:brightness-125"
        >create a backup</AutoLink
      >
      before proceeding, as this process is irreversible and may permanently alter your world or the
      files on your server.
    </span>
  </div>
</template>
