<template>
  <div
    class="markdown-body"
    @click="handleLinkClick"
    v-html="renderHighlightedString(description ?? '')"
  />
  <ExternalLinkModal ref="externalLinkModal" />
</template>

<script setup lang="ts">
import { renderHighlightedString } from '@modrinth/utils'
import { ref } from 'vue'
import ExternalLinkModal from '../modal/ExternalLinkModal.vue'

const externalLinkModal = ref<InstanceType<typeof ExternalLinkModal> | null>(null)

const handleLinkClick = async (e: MouseEvent) => {
  const link = (e.target as HTMLElement)?.closest('a')
  if (!link) return

  const href = link.getAttribute('href')
  if (!href) return

  try {
    const url = new URL(href)
    if (!url.hostname.includes('modrinth.com') && url.host !== window.location.host) {
      e.preventDefault()
      const proceed = await externalLinkModal.value?.show(href)
      if (proceed) {
        window.open(href, '_blank', 'noopener,noreferrer')
      }
    }
  } catch {
    // not sure how this'd happen. but if it does, just ignore it
    // and let the default behavior happen
    // e.g. if the link is a mailto link or something
    // this is a bit of a hack, but it works
    return
  }
}

withDefaults(
  defineProps<{
    description: string
  }>(),
  {},
)
</script>
