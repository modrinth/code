<template>
  <header
    data-pyro-files-state="editing"
    class="flex h-12 select-none items-center justify-between rounded-t-2xl bg-table-alternateRow p-3"
    aria-label="File editor navigation"
  >
    <nav
      aria-label="Breadcrumb navigation"
      class="m-0 flex min-w-0 flex-shrink items-center p-0 text-contrast"
    >
      <ol class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
        <li class="-ml-1 flex-shrink-0">
          <ButtonStyled type="transparent">
            <button
              v-tooltip="'Back to home'"
              type="button"
              class="mr-2 grid h-12 w-10 place-content-center focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
              @click="goHome"
            >
              <span
                class="grid size-8 place-content-center rounded-full bg-button-bg p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
              >
                <HomeIcon class="h-5 w-5" />
                <span class="sr-only">Home</span>
              </span>
            </button>
          </ButtonStyled>
        </li>
        <li class="m-0 -ml-2 p-0">
          <ol class="m-0 flex items-center p-0">
            <li
              v-for="(segment, index) in breadcrumbSegments"
              :key="index"
              class="flex items-center text-sm"
            >
              <ButtonStyled type="transparent">
                <button
                  class="cursor-pointer focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
                  :class="{ '!text-contrast': index === breadcrumbSegments.length - 1 }"
                  @click="$emit('navigate', index)"
                >
                  {{ segment || '' }}
                </button>
              </ButtonStyled>
              <ChevronRightIcon
                v-if="index < breadcrumbSegments.length"
                class="size-4 text-secondary"
                aria-hidden="true"
              />
            </li>
            <li class="flex items-center px-3 text-sm">
              <span class="font-semibold !text-contrast" aria-current="location">{{
                fileName
              }}</span>
            </li>
          </ol>
        </li>
      </ol>
    </nav>
    <div v-if="!isImage" class="flex gap-2">
      <Button
        v-if="isLogFile"
        v-tooltip="'Share to mclo.gs'"
        icon-only
        transparent
        aria-label="Share to mclo.gs"
        @click="$emit('share')"
      >
        <ShareIcon />
      </Button>
      <ButtonStyled type="transparent">
        <TeleportOverflowMenu
          position="bottom"
          direction="left"
          aria-label="Save file"
          :options="[
            { id: 'save', action: () => $emit('save') },
            { id: 'save-as', action: () => $emit('save-as') },
            { id: 'save&restart', action: () => $emit('save-restart') },
          ]"
        >
          <SaveIcon aria-hidden="true" />
          <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
          <template #save> <SaveIcon aria-hidden="true" /> Save </template>
          <template #save-as> <SaveIcon aria-hidden="true" /> Save as... </template>
          <template #save&restart>
            <RefreshClockwiseIcon aria-hidden="true" />
            Save & restart
          </template>
        </TeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </header>
</template>

<script setup lang="ts">
import { DropdownIcon, SaveIcon, ShareIcon, HomeIcon, ChevronRightIcon, RefreshClockwiseIcon } from '@modrinth/assets'
import { computed, defineProps, defineEmits } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ButtonStyled, TeleportOverflowMenu, Button } from "@modrinth/ui"

const props = defineProps<{
  breadcrumbSegments: string[]
  fileName?: string
  isImage: boolean
  filePath?: string
}>()

const isLogFile = computed(() => {
  return props.filePath?.startsWith('logs') || props.filePath?.endsWith('.log')
})

const route = useRoute()
const router = useRouter()

const emit = defineEmits<{
  (e: 'cancel' | 'save' | 'save-as' | 'save-restart' | 'share'): void
  (e: 'navigate', index: number): void
}>()

const goHome = () => {
  emit('cancel')
  router.push({ path: '/servers/manage/' + route.params.id + '/files' })
}
</script>
