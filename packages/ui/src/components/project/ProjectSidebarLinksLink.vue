<template>
  <a
    v-if="url"
    :href="url"
    :target="linkTarget"
    class="flex gap-2 items-center w-fit text-primary leading-[1.2] hover:underline"
    rel="noopener nofollow ugc"
  >
    <slot />
    <ExternalIcon aria-hidden="true" class="external-icon" />
  </a>
</template>
<script setup lang="ts">
import {
  ExternalIcon,
} from '@modrinth/assets'

defineProps<{
  url?: string
  linkTarget: string
}>()
</script>
