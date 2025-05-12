<script setup lang="ts">
import { computedAsync } from '@vueuse/core'
import { ref, defineProps, defineExpose, nextTick } from 'vue'
import {
  ContractIcon,
  ExpandIcon,
  ExternalIcon,
  LeftArrowIcon,
  RightArrowIcon,
  XIcon,
} from '@modrinth/assets'
import ButtonStyled from '../base/ButtonStyled.vue'
import Button from '../base/Button.vue'
import { pxOf } from '@modrinth/utils'

export type GalleryEntry = {
  src: string
  key: any | GalleryKey
  alt: string
}
export type NavigationFunction = (key: any | GalleryKey) => GalleryEntry | Promise<GalleryEntry>
export type OpenExternallyFunction = (src: string, key: any | GalleryKey) => void | Promise<void>
export type GalleryKey = { title?: string; description?: string }

const props = withDefaults(
  defineProps<{
    next: NavigationFunction
    prev: NavigationFunction
    openExternally: OpenExternallyFunction
    openExternallyTooltip?: string
    disableZoom?: boolean
  }>(),
  {
    disableZoom: false,
    openExternallyTooltip: 'Open externally',
  },
)

const src = ref<string>()
const alt = ref<string>()
const key = ref<any | GalleryKey>()
const shown = ref(false)
const scale = ref(1)
const imageRef = ref<HTMLImageElement | null>(null)

function show(_src: string, _alt: string, _key: any | GalleryKey) {
  shown.value = true
  src.value = _src
  alt.value = _alt
  key.value = _key
}
function hide() {
  shown.value = false
}

const nextImageData = computedAsync(() => props.next(key.value))
const prevImageData = computedAsync(() => props.prev(key.value))

async function nextImage() {
  const data = nextImageData.value
  if (data) show(data.src, data.alt, data.key)
}
async function prevImage() {
  const data = prevImageData.value
  if (data) show(data.src, data.alt, data.key)
}
async function handleOpenExternally() {
  if (src.value && key.value) await props.openExternally(src.value, key.value)
}

async function toggleZoom() {
  if (scale.value === 1 && imageRef.value) {
    await nextTick()

    const vpW = window.innerWidth
    const vpH = window.innerHeight

    const gapPx = pxOf('--gap-lg')

    const rect = imageRef.value.getBoundingClientRect()
    const baseW = rect.width
    const baseH = rect.height

    const maxScaleW = (vpW - 2 * gapPx) / baseW
    const maxScaleH = (vpH - 2 * gapPx) / baseH

    scale.value = Math.min(maxScaleW, maxScaleH, 3)
  } else {
    scale.value = 1
  }
}

defineExpose({ show, hide })
</script>

<template>
  <div
    v-if="shown"
    class="fixed inset-0 z-20 flex items-center justify-center bg-black/70"
    @click="hide"
  >
    <img
      v-if="src"
      ref="imageRef"
      :src="src"
      :alt="alt"
      class="object-contain origin-[center_center] transform transition-transform duration-300 ease-in-out rounded-[var(--radius-lg)]"
      style="
        max-width: calc(100vw - 2 * var(--gap-lg));
        max-height: calc(100vh - 2 * var(--gap-lg));
      "
      :style="{ transform: `scale(${scale})` }"
      @click.stop
    />

    <div
      class="fixed left-1/2 bottom-4 -translate-x-1/2 flex flex-col items-center gap-[var(--gap-sm)] transition-all duration-250 ease-in-out group"
      @click.stop
    >
      <div
        v-if="key?.title || key?.description"
        class="flex flex-col max-w-[40rem] mb-1 gap-2 transition-all duration-250 ease-in-out group-hover:opacity-100 opacity-0 group-hover:translate-y-0 translate-y-5 group-hover:scale-100 scale-80"
      >
        <h2
          v-if="key?.title"
          class="text-shadow text-[var(--dark-color-text-dark)] text-xl text-center m-0"
        >
          {{ key.title }}
        </h2>
        <p
          v-if="key?.description"
          class="text-shadow text-[var(--dark-color-text)] m-0 text-center"
        >
          {{ key.description }}
        </p>
      </div>

      <div
        class="card !p-3 transition-all duration-250 ease-in-out group-hover:opacity-100 opacity-40 group-hover:translate-y-0 translate-y-1 group-hover:scale-100 scale-90"
      >
        <div class="flex items-center gap-2">
          <ButtonStyled v-tooltip="'Close'" circular icon-only @click="hide">
            <Button><XIcon /><span class="sr-only">Close</span></Button>
          </ButtonStyled>

          <ButtonStyled
            v-tooltip="openExternallyTooltip"
            circular
            icon-only
            @click="handleOpenExternally"
          >
            <Button
              ><ExternalIcon /><span class="sr-only">{{ openExternallyTooltip }}</span></Button
            >
          </ButtonStyled>

          <template v-if="!disableZoom">
            <ButtonStyled v-tooltip="'Toggle zoom'" circular icon-only @click="toggleZoom">
              <Button>
                <ExpandIcon v-if="scale <= 1" /><ContractIcon v-else />
                <span class="sr-only">Toggle zoom</span>
              </Button>
            </ButtonStyled>
          </template>

          <template v-if="prevImageData">
            <ButtonStyled v-tooltip="'Previous'" circular icon-only @click="prevImage">
              <Button><LeftArrowIcon /><span class="sr-only">Previous</span></Button>
            </ButtonStyled>
          </template>

          <template v-if="nextImageData">
            <ButtonStyled v-tooltip="'Next'" circular icon-only @click="nextImage">
              <Button><RightArrowIcon /><span class="sr-only">Next</span></Button>
            </ButtonStyled>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.text-shadow {
  filter: drop-shadow(0 0 2px rgba(0, 0, 0, 1)) drop-shadow(0 0 4px rgba(0, 0, 0, 1));
}
</style>
