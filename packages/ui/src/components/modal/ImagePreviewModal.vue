<script setup lang="ts">
import { computedAsync } from '@vueuse/core'
import { ref} from "vue";
import { ContractIcon, ExpandIcon, ExternalIcon, LeftArrowIcon, RightArrowIcon, XIcon } from "@modrinth/assets";
import ButtonStyled from "../base/ButtonStyled.vue";
import Button from "../base/Button.vue";

export type GalleryEntry = {
  src: string,
  key: any | GalleryKey,
  alt: string
};

export type NavigationFunction = (key: any) => GalleryEntry | Promise<GalleryEntry>;
export type OpenExternallyFunction = (src: string, key: any) => (void | Promise<void>);

export type GalleryKey = {
  title?: string,
  description?: string;
}

const props = defineProps<{
  next: NavigationFunction,
  prev: NavigationFunction,
  openExternally: OpenExternallyFunction,
  disableZoom: boolean
}>();

const src = ref<string | undefined>(undefined);
const alt = ref<string | undefined>(undefined);
const key = ref<any | GalleryKey | undefined>(undefined);
const shown = ref(false);
const zoomedIn = ref(false);

function show(_src: string, _alt: string, _key: any | GalleryKey) {
  shown.value = true;
  src.value = _src;
  alt.value = _alt;
  key.value = _key;
}

function hide() {
  shown.value = false;
  src.value = undefined;
  alt.value = undefined;
  key.value = undefined;
}

const nextImageData = computedAsync(() => {
  return props.next(key.value);
})

const prevImageData = computedAsync(() => {
  return props.prev(key.value);
})

async function nextImage() {
  const data = nextImageData.value;
  if (!data) return;

  show(data.src, data.alt, data.key);
}

async function prevImage() {
  const data = prevImageData.value;
  if (!data) return;

  show(data.src, data.alt, data.key);
}

async function handleOpenExternally() {
  if (src.value && key.value) {
    await props.openExternally(src.value, key.value);
  }
}

defineExpose({ show, hide });
</script>

<template>
  <div
      v-if="shown"
      class="fixed inset-0 z-20 flex items-center justify-center bg-black/70"
      @click="hide"
  >
    <div
        class="relative w-[calc(100vw-2*var(--spacing-card-lg))] h-[calc(100vh-2*var(--spacing-card-lg))]"
        @click.stop
    >
      <img
          v-if="src"
          :src="src"
          :alt="alt"
          class=""
          :class="[
            zoomedIn && 'object-cover w-auto h-[calc(100vh-2*var(--spacing-card-lg))]',
            'absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2',
            'max-w-[calc(100vw-2*var(--spacing-card-lg))] max-h-[calc(100vh-2*var(--spacing-card-lg))]',
            'rounded-[var(--size-rounded-card)]'
          ]"
      />
    </div>

    <div
        class="fixed left-1/2 bottom-4
             -translate-x-1/2 flex flex-col items-center
             gap-[var(--spacing-card-sm)]
             transition-all duration-250 ease-in-out group"
        @click.stop
    >
      <div
          v-if="key?.title || key?.description"
          class="flex flex-col max-w-[40rem] text-shadow mb-1 gap-2
               transition-all duration-250 ease-in-out
               group-hover:opacity-100 opacity-0
               group-hover:translate-y-0 translate-y-5
               group-hover:scale-100 scale-80"
      >
        <h2
            v-if="key?.title"
            class="text-[var(--dark-color-text-dark)] text-xl text-center m-0"
        >
          {{ key.title }}
        </h2>
        <p
            v-if="key?.description"
            class="text-[var(--dark-color-text)] m-0 text-center"
        >
          {{ key.description }}
        </p>
      </div>

      <div
          class="card !p-3
               transition-all duration-250 ease-in-out
               group-hover:opacity-100 opacity-40
               group-hover:translate-y-0 translate-y-1
               group-hover:scale-100 scale-90"
      >
        <div class="flex flex-row items-center gap-2">
          <ButtonStyled circular icon-only @click="hide">
            <Button>
              <XIcon aria-hidden="true" />
              <span class="sr-only">Close</span>
            </Button>
          </ButtonStyled>

          <ButtonStyled circular icon-only @click="handleOpenExternally">
            <Button>
              <ExternalIcon aria-hidden="true" />
              <span class="sr-only">Open externally</span>
            </Button>
          </ButtonStyled>

          <template v-if="!disableZoom">
            <ButtonStyled circular icon-only @click="zoomedIn = !zoomedIn">
              <Button>
                <ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
                <ContractIcon v-else aria-hidden="true" />
                <span class="sr-only">Toggle zoom</span>
              </Button>
            </ButtonStyled>
          </template>

          <template v-if="prevImageData">
            <ButtonStyled circular icon-only @click="prevImage">
              <Button>
                <LeftArrowIcon aria-hidden="true" />
                <span class="sr-only">Previous image</span>
              </Button>
            </ButtonStyled>
          </template>


          <template v-if="nextImageData">
            <ButtonStyled circular icon-only @click="nextImage">
              <Button>
                <RightArrowIcon aria-hidden="true" />
                <span class="sr-only">Next image</span>
              </Button>
            </ButtonStyled>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.text-shadow {
  text-shadow: 1px 1px 10px rgba(0, 0, 0, 0.83);
}
</style>