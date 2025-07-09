<script setup lang="ts">
import { useTemplateRef, ref, computed } from 'vue'
import type { Cape, SkinModel } from '@/helpers/skins.ts'
import {
  ButtonStyled,
  ScrollablePanel,
  CapeButton,
  CapeLikeTextButton,
  SkinPreviewRenderer,
} from '@modrinth/ui'
import { CheckIcon, XIcon } from '@modrinth/assets'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const modal = useTemplateRef('modal')

const emit = defineEmits<{
  (e: 'select', cape: Cape | undefined): void
  (e: 'cancel'): void
}>()

const props = defineProps<{
  capes: Cape[]
}>()

const sortedCapes = computed(() => {
  return [...props.capes].sort((a, b) => {
    const nameA = (a.name || '').toLowerCase()
    const nameB = (b.name || '').toLowerCase()
    return nameA.localeCompare(nameB)
  })
})

const currentSkinId = ref<string | undefined>()
const currentSkinTexture = ref<string | undefined>()
const currentSkinVariant = ref<SkinModel>('CLASSIC')
const currentCapeTexture = computed<string | undefined>(() => currentCape.value?.texture)
const currentCape = ref<Cape | undefined>()

function show(
  e: MouseEvent,
  skinId?: string,
  selected?: Cape,
  skinTexture?: string,
  variant?: SkinModel,
) {
  currentSkinId.value = skinId
  currentSkinTexture.value = skinTexture
  currentSkinVariant.value = variant || 'CLASSIC'
  currentCape.value = selected
  modal.value?.show(e)
}

function select() {
  emit('select', currentCape.value)
  hide()
}

function hide() {
  modal.value?.hide()
  emit('cancel')
}

function updateSelectedCape(cape: Cape | undefined) {
  currentCape.value = cape
}

function onModalHide() {
  emit('cancel')
}

defineExpose({
  show,
  hide,
})
</script>
<template>
  <ModalWrapper ref="modal" @on-hide="onModalHide">
    <template #title>
      <div class="flex flex-col">
        <span class="text-lg font-extrabold text-heading">Change cape</span>
      </div>
    </template>
    <div class="flex flex-col md:flex-row gap-6">
      <div class="max-h-[25rem] h-[25rem] w-[16rem] min-w-[16rem] overflow-hidden relative">
        <div class="absolute top-[-4rem] left-0 h-[32rem] w-[16rem] flex-shrink-0">
          <SkinPreviewRenderer
            v-if="currentSkinTexture"
            :cape-src="currentCapeTexture"
            :texture-src="currentSkinTexture"
            :variant="currentSkinVariant"
            :scale="1.4"
            :fov="50"
            :initial-rotation="Math.PI + Math.PI / 8"
            class="h-full w-full"
          />
        </div>
      </div>

      <div class="flex flex-col gap-4 w-full my-auto">
        <ScrollablePanel class="max-h-[20rem] max-w-[30rem] mb-5 h-full">
          <div class="flex flex-wrap gap-2 justify-center content-start overflow-y-auto h-full">
            <CapeLikeTextButton
              tooltip="No Cape"
              :highlighted="!currentCape"
              @click="updateSelectedCape(undefined)"
            >
              <template #icon>
                <XIcon />
              </template>
              <span>None</span>
            </CapeLikeTextButton>
            <CapeButton
              v-for="cape in sortedCapes"
              :id="cape.id"
              :key="cape.id"
              :name="cape.name"
              :texture="cape.texture"
              :selected="currentCape?.id === cape.id"
              @select="updateSelectedCape(cape)"
            />
          </div>
        </ScrollablePanel>
      </div>
    </div>
    <div class="flex gap-2 items-center">
      <ButtonStyled color="brand">
        <button @click="select">
          <CheckIcon />
          Select
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide">
          <XIcon />
          Cancel
        </button>
      </ButtonStyled>
    </div>
  </ModalWrapper>
</template>
