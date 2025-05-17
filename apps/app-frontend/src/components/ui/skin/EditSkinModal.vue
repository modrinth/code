<template>
  <ModalWrapper ref="modal" @on-hide="resetState">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">
        {{ mode === 'edit' ? 'Edit skin' : 'New skin' }}
      </span>
    </template>

    <div class="flex flex-col md:flex-row gap-6">
      <div class="max-h-[25rem] w-[16rem] min-w-[16rem] overflow-hidden relative">
        <div class="absolute top-[-4rem] left-0 h-[32rem] w-[16rem] flex-shrink-0">
          <SkinPreviewRenderer
            slim-model-src="/src/assets/models/slim_player.gltf"
            wide-model-src="/src/assets/models/classic_player.gltf"
            cape-model-src="/src/assets/models/cape.gltf"
            :variant="variant"
            :texture-src="previewSkin"
            :cape-src="selectedCapeTexture"
            :scale="1.4" :fov="50"
            :initial-rotation="Math.PI / 8"
            class="h-full w-full"
          />
        </div>
      </div>

      <div class="flex flex-col gap-4 w-full">
        <section>
          <h2 class="text-base font-semibold mb-2">Texture</h2>
          <Card class="!bg-bg p-4 flex flex-col items-center gap-4">
            <FileInput
              :max-size="8000"
              accept="image/png"
              :prompt="mode === 'edit' ? 'Replace skin' : 'Upload a skin'"
              class="btn btn-primary"
              @change="onTextureSelected"
            >
              <UploadIcon aria-hidden="true" />
            </FileInput>

            <div class="flex items-center gap-2" v-if="fileName || mode === 'new'">
              <InfoIcon v-if="!fileName" class="text-brand-blue" />
              <CheckCircleIcon v-else class="text-brand-green" />
              <small class="truncate" v-tooltip="fileName">{{ fileName || 'No skin uploaded yet.' }}</small>
            </div>
          </Card>
        </section>

        <section>
          <h2 class="text-base font-semibold mb-2">Arm style</h2>
          <RadioButtons v-model="variant" :items="['CLASSIC','SLIM']">
            <template #default="{ item }">
              {{ item === 'CLASSIC' ? 'Wide' : 'Slim' }}
            </template>
          </RadioButtons>
        </section>

        <section>
          <h2 class="text-base font-semibold mb-2">Cape</h2>
          <div class="flex gap-2">
            <CapeLikeTextButton
              tooltip="No Cape"
              :highlighted="!selectedCape"
              @click="selectCape(undefined)"
            >
              <template #icon>
                <XIcon />
              </template>
              <span>No Cape</span>
            </CapeLikeTextButton>

            <CapeButton
              v-for="cape in visibleCapeList"
              :key="cape.id"
              :id="cape.id"
              :texture="cape.texture"
              :name="cape.name || 'Cape'"
              :selected="selectedCape?.id === cape.id"
              @select="selectCape(cape)"
            />

            <CapeLikeTextButton
              v-if="(capes?.length ?? 0) > 2"
              tooltip="View more capes"
              @mouseup="openSelectCapeModal"
            >
              <template #icon><ChevronRightIcon/></template>
              <span>More</span>
            </CapeLikeTextButton>
          </div>
        </section>
      </div>
    </div>

    <div class="flex gap-2 mt-6">
      <ButtonStyled color="brand" :disabled="disableSave">
        <Button @click="save" :disabled="disableSave" v-tooltip="disableSave && 'Upload a skin first!'">
          <CheckIcon v-if="mode === 'new'" /><SaveIcon v-else />
          {{ mode === 'new' ? 'Add skin' : 'Save skin' }}
        </Button>
      </ButtonStyled>
      <Button @click="hide"><XIcon/>Cancel</Button>
      <ButtonStyled color="red" v-if="mode==='edit'">
        <Button @click="handleDelete"><TrashIcon/>Delete</Button>
      </ButtonStyled>
    </div>
  </ModalWrapper>

  <SelectCapeModal ref="selectCapeModal" :capes="capes || []" @select="handleCapeSelected" />
</template>

<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import {
  Card, FileInput, SkinPreviewRenderer,
  Button, RadioButtons, CapeButton, CapeLikeTextButton, ButtonStyled
} from '@modrinth/ui'
import {
  add_and_equip_custom_skin,
  remove_custom_skin,
  equip_skin,
  unequip_skin,
  get_available_skins,
  type Skin, type Cape, type SkinModel
} from '@/helpers/skins.ts'
import { handleError } from '@/store/notifications'
import {
  CheckCircleIcon, InfoIcon, UploadIcon,
  CheckIcon, SaveIcon, XIcon, TrashIcon, ChevronRightIcon
} from '@modrinth/assets'

const modal = useTemplateRef('modal')
const selectCapeModal = useTemplateRef('selectCapeModal')
const mode = ref<'new'|'edit'>('new')
const currentSkin = ref<Skin|null>(null)
const shouldRestoreModal = ref(false)

const fileUploadTextureBlob = ref<Uint8Array|null>(null)
const fileName = ref<string|null>(null)
watch(fileUploadTextureBlob, () => {
  if (fileName.value === null && fileUploadTextureBlob.value) {
    fileName.value = 'New upload'
  }
})

const variant = ref<SkinModel>('CLASSIC')
const selectedCape = ref<Cape|undefined>(undefined)
const props = defineProps<{ capes?: Cape[] }>()

const selectedCapeTexture = computed(() => selectedCape.value?.texture)
const visibleCapeList = ref<Cape[]>([])

function initVisibleCapeList() {
  if (!props.capes || props.capes.length === 0) {
    visibleCapeList.value = []
    return
  }

  if (visibleCapeList.value.length === 0) {
    if (selectedCape.value) {
      const otherCape = getRandomCapeExcluding(selectedCape.value.id)
      visibleCapeList.value = otherCape ? [selectedCape.value, otherCape] : [selectedCape.value]
    } else {
      visibleCapeList.value = getRandomCapes(2)
    }
  }
}

function getRandomCapes(count: number): Cape[] {
  if (!props.capes || props.capes.length === 0) return []

  const availableCapes = [...props.capes]
  const result: Cape[] = []

  for (let i = 0; i < count && availableCapes.length > 0; i++) {
    const randomIndex = Math.floor(Math.random() * availableCapes.length)
    const cape = availableCapes.splice(randomIndex, 1)[0]
    result.push(cape)
  }

  return result
}

function getRandomCapeExcluding(excludeId: string): Cape | undefined {
  if (!props.capes || props.capes.length <= 1) return undefined

  const availableCapes = props.capes.filter(cape => cape.id !== excludeId)
  if (availableCapes.length === 0) return undefined

  const randomIndex = Math.floor(Math.random() * availableCapes.length)
  return availableCapes[randomIndex]
}

const localPreviewUrl = ref<string|null>(null)
watch(fileUploadTextureBlob, (blob, prev) => {
  if (prev && localPreviewUrl.value) URL.revokeObjectURL(localPreviewUrl.value)
  if (blob) localPreviewUrl.value = URL.createObjectURL(new Blob([blob]))
  else localPreviewUrl.value = null
})
const previewSkin = computed(() => {
  if (localPreviewUrl.value) return localPreviewUrl.value
  if (currentSkin.value) return currentSkin.value.texture
  return '/src/assets/skins/steve.png'
})

const disableSave = computed(() =>
  mode.value==='new' && !fileUploadTextureBlob.value
)

function resetState() {
  mode.value = 'new'
  currentSkin.value = null
  fileUploadTextureBlob.value = null
  fileName.value = null
  variant.value = 'CLASSIC'
  selectedCape.value = undefined
  visibleCapeList.value = []
  shouldRestoreModal.value = false
  if (localPreviewUrl.value) {
    URL.revokeObjectURL(localPreviewUrl.value)
    localPreviewUrl.value = null
  }
}

function show(e: MouseEvent, skin?: Skin) {
  mode.value = skin ? 'edit' : 'new'
  currentSkin.value = skin ?? null
  if (skin) {
    variant.value = skin.variant
    selectedCape.value = props.capes?.find(c=>c.id===skin.cape_id)
  } else {
    variant.value = 'CLASSIC'
    selectedCape.value = undefined
  }
  visibleCapeList.value = []
  initVisibleCapeList()
  modal.value?.show(e)
}

function hide() {
  modal.value?.hide()
  resetState()
}

async function onTextureSelected(files: FileList|null) {
  if (!files?.length) return
  const file = files[0]
  fileName.value = file.name
  const buf = await file.arrayBuffer()
  fileUploadTextureBlob.value = new Uint8Array(buf)
}

function changeVariant(v: SkinModel){ variant.value = v }

function selectCape(cape: Cape|undefined) {
  if (cape && selectedCape.value?.id !== cape.id) {
    const isInVisibleList = visibleCapeList.value.some(c => c.id === cape.id)
    if (!isInVisibleList && visibleCapeList.value.length > 0) {
      visibleCapeList.value.splice(0, 1, cape)
    }
  }
  selectedCape.value = cape
}

function handleCapeSelected(cape: Cape | undefined) {
  selectCape(cape)

  if (shouldRestoreModal.value) {
    setTimeout(() => {
      modal.value?.show()
      shouldRestoreModal.value = false
    }, 0)
  }
}

function openSelectCapeModal(e: MouseEvent) {
  if (!selectCapeModal.value) return

  shouldRestoreModal.value = true
  modal.value?.hide()

  setTimeout(() => {
    selectCapeModal.value?.show(
      e,
      currentSkin.value?.texture_key,
      selectedCape.value,
      previewSkin.value,
      variant.value
    )
  }, 0)
}

async function handleDelete() {
  try {
    if (currentSkin.value?.is_equipped) {
      await unequip_skin()
    }
    await remove_custom_skin(currentSkin.value!)
    emit('deleted', currentSkin.value!)
    hide()
  } catch (err) {
    handleError(err)
  }
}

async function save() {
  try {
    let blob: Uint8Array
    if (fileUploadTextureBlob.value) {
      blob = fileUploadTextureBlob.value
    } else {
      const url = currentSkin.value!.texture
      const resp = await fetch(url)
      const buf = await resp.arrayBuffer()
      blob = new Uint8Array(buf)
    }

    await unequip_skin();

    if (mode.value === 'new') {
      await add_and_equip_custom_skin(blob, variant.value, selectedCape.value)
      emit('saved')
    } else {
      await add_and_equip_custom_skin(blob, variant.value, selectedCape.value)
      await remove_custom_skin(currentSkin.value!)
      emit('saved')
    }

    hide()
  } catch (err) {
    handleError(err)
  }
}

watch(() => props.capes, () => {
  initVisibleCapeList()
}, { immediate: true })

const emit = defineEmits<{
  (e:'saved'):void
  (e:'deleted', skin: Skin):void
}>()

defineExpose({ show, hide })
</script>
