<template>
  <div
    class="grid grid-cols-[min-content_auto_min-content_min-content] items-center gap-2 rounded-2xl border-[1px] border-divider bg-bg p-2"
  >
    <VersionChannelIndicator :channel="version.version_type" />
    <div class="flex min-w-0 flex-col gap-1">
      <h1 class="my-0 truncate text-nowrap text-base font-extrabold leading-none text-contrast">
        {{ version.version_number }}
      </h1>
      <p class="m-0 truncate text-nowrap text-xs font-semibold text-secondary">
        {{ version.name }}
      </p>
    </div>
    <ButtonStyled color="brand">
      <a :href="downloadUrl" class="min-w-0" @click="emit('onDownload')">
        <DownloadIcon aria-hidden="true" /> Download
      </a>
    </ButtonStyled>
    <ButtonStyled circular>
      <nuxt-link
        :to="`/project/${props.version.project_id}/version/${props.version.id}`"
        class="min-w-0"
        aria-label="Open project page"
        @click="emit('onNavigate')"
      >
        <ExternalIcon aria-hidden="true" />
      </nuxt-link>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled, VersionChannelIndicator } from '../index'
import { DownloadIcon, ExternalIcon } from '@modrinth/assets'
import { computed } from 'vue'

const props = defineProps<{
  version: Version
}>()

const downloadUrl = computed(() => {
  const primary: VersionFile = props.version.files.find((x) => x.primary) || props.version.files[0]
  return primary.url
})

const emit = defineEmits(['onDownload', 'onNavigate'])
</script>
