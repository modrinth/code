<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { useTemplateRef, ref, computed } from 'vue'
import CapeButton from '@/components/ui/skin/CapeButton.vue'
import type { Cape } from '@/helpers/skins.ts'
import { ButtonStyled, ScrollablePanel } from '@modrinth/ui'
import { CheckIcon, XIcon} from '@modrinth/assets'

const modal = useTemplateRef('modal')

const emit = defineEmits<{
  (e: 'select', cape: string | undefined): void
}>()

const props = defineProps<{
  capes: Cape[];
}>()


const currentSkin = ref<string | undefined>()
const currentCape = ref<Cape | undefined>()

const mcUrlRegex = /texture\/([a-fA-F0-9]+)$/

const textureId = computed(() => {
  if (!currentCape.value) {
    return undefined
  }

  const mcTextureMatch = currentCape.value.texture.match(mcUrlRegex)

  if (mcTextureMatch) {
    return mcTextureMatch[1]
  } else {
    return undefined;
  }
})


const capeParams = computed(() => {
  return textureId.value ? `&replacecape=${textureId.value}` : ``
})

const previewSkin = computed(() => currentSkin.value ? `https://vzge.me/full/350/${currentSkin.value}.png?no=ears&y=180${capeParams.value}` : undefined)

function show(e: MouseEvent, skin?: string, selected?: Cape) {
  currentSkin.value = skin
  currentCape.value = selected
  modal.value?.show(e)
}

function select() {
  emit('select', currentCape.value?.id)
  hide()
}

function hide() {
  modal.value?.hide()
}

defineExpose({
  show,
  hide,
})
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">Selecting a cape</span>
    </template>
    <div class="grid grid-cols-[auto_1fr] gap-6">
      <div class="flex">
        <img :src="previewSkin" alt="" class="w-auto my-auto h-60 object-contain" />
      </div>
      <div>
        <ScrollablePanel class="h-[20rem] w-[30rem]">
          <div class="grid grid-cols-6 gap-2 items-start w-full">
            <CapeButton v-for="cape in capes" :key="`cape-${cape.id}`" :cape="cape" :selected="currentCape?.id === cape.id" @select="currentCape = cape" />
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
