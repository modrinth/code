<template>
  <ModalWrapper ref="modal" @on-modal-hide="resetState">
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
            :variant="variant"
            :texture-src="previewSkin"
            :scale="1.4" :fov="50"
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
            <CapeButton
              v-for="cape in firstThreeCapes"
              :key="cape.id"
              :id="cape.id"
              :texture="cape.texture"
              :name="cape.name || 'Cape'"
              :selected="selectedCape?.id === cape.id"
              @select="selectCape(cape)"
            />
            <CapeLikeTextButton v-if="capes?.length ?? 0 > 3" tooltip="View more capes" @click="viewMoreCapes">
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
</template>

<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
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
const mode = ref<'new'|'edit'>('new')
const currentSkin = ref<Skin|null>(null)

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
const firstThreeCapes = computed(() => props.capes?.slice(0,3) ?? [])

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
function selectCape(c: Cape|undefined){ selectedCape.value = c }
function viewMoreCapes(){
  // TODO: later
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

const emit = defineEmits<{
  (e:'saved'):void
  (e:'deleted'):void
}>()

defineExpose({ show, hide })
</script>
