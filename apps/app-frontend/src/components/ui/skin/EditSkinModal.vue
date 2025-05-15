<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import {
  Card,
  Chips,
  FileInput,
  SkinPreviewRenderer,
  CapeButton,
  CapeLikeTextButton,
  Button, RadioButtons
} from '@modrinth/ui'
import {
  add_and_equip_custom_skin,
  type Cape,
  determineModelType,
  equip_skin,
  type Skin,
  type SkinModel
} from '@/helpers/skins.ts'
import {handleError} from '@/store/notifications'
import {computed, ref, useTemplateRef, watch} from 'vue'
import {CheckCircleIcon, InfoIcon, UploadIcon, ChevronRightIcon} from "@modrinth/assets";

const modal = useTemplateRef('modal')
const mode = ref<'new' | 'edit'>('new')
const currentSkin = ref<Skin | null>(null)
const textureBlob = ref<Uint8Array | null>(null)
const variant = ref<SkinModel>('Classic')
const selectedCape = ref<Cape | undefined>(undefined)

const props = defineProps<{ capes?: Cape[] }>()
const firstThreeCapes = computed(() => props.capes?.slice(0, 3) ?? [])

const localPreviewUrl = ref<string | null>(null)
const fileName = ref<string | null>(null)

watch(textureBlob, async (blob, prev) => {
  if (prev && localPreviewUrl.value) URL.revokeObjectURL(localPreviewUrl.value)
  localPreviewUrl.value = blob ? URL.createObjectURL(new Blob([blob])) : null

  if (blob && variant.value === 'Unknown' && localPreviewUrl.value) {
    try {
      variant.value = await determineModelType(localPreviewUrl.value)
    } catch (err) {
      handleError(err)
    }
  }
})

const previewSkin = computed(() => {
  if (localPreviewUrl.value) return localPreviewUrl.value
  if (currentSkin.value) return currentSkin.value.texture;
  return ''
})

const emit = defineEmits<{
  (e: 'view-more-capes', ev: MouseEvent): void
  (e: 'saved'): void
}>()

function resetState() {
  mode.value         = 'new'
  currentSkin.value  = null
  textureBlob.value  = null
  fileName.value     = null

  if (localPreviewUrl.value) {
    URL.revokeObjectURL(localPreviewUrl.value)
    localPreviewUrl.value = null
  }

  variant.value      = 'Classic'
  selectedCape.value = undefined
}

function show(e: MouseEvent, skin?: Skin) {
  mode.value = skin ? 'edit' : 'new'
  currentSkin.value = skin ?? null
  textureBlob.value = null
  variant.value = skin?.variant ?? 'Classic'
  selectedCape.value = skin?.cape_id ? props.capes?.find(c => c.id === skin.cape_id) : undefined
  modal.value?.show(e)
}

function hide() {
  modal.value?.hide()
  resetState()
}

async function onTextureSelected(files: FileList | null) {
  if (!files?.length) return
  const file = files[0]
  const buf = await file.arrayBuffer()
  textureBlob.value = new Uint8Array(buf)
  fileName.value = file.name
}

function changeVariant(newVariant: SkinModel) {
  variant.value = newVariant
}

function selectCape(cape: Cape | undefined) {
  selectedCape.value = cape
}

function viewMoreCapes(e: MouseEvent) {
  hide()
  emit('view-more-capes', e)
}

async function save() {
  try {
    if (mode.value === 'new') {
      if (!textureBlob.value) throw new Error('Please upload a skin texture first.')
      await add_and_equip_custom_skin(textureBlob.value, variant.value, selectedCape.value).catch(handleError)
    } else if (currentSkin.value) {
      const edited: Skin = {
        ...currentSkin.value,
        variant: variant.value,
        cape_id: selectedCape.value?.id
      }
      if (textureBlob.value) {
        await add_and_equip_custom_skin(textureBlob.value, variant.value, selectedCape.value).catch(handleError)
      } else {
        await equip_skin(edited).catch(handleError)
      }
    }
    hide()
    emit('saved')
  } catch (err) {
    handleError(err)
  }
}

defineExpose({
  show,
  hide,
  save,
  onTextureSelected,
  changeVariant,
  selectCape,
  viewMoreCapes,
})
</script>

<template>
  <ModalWrapper ref="modal" @on-modal-hide="resetState">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">{{
          mode === 'edit' ? 'Edit skin' : 'New skin'
        }}</span>
    </template>
    <div class="flex flex-col md:flex-row gap-6">
      <div class="max-h-[25rem] w-[16rem] min-w-[16rem] overflow-hidden relative">
        <!-- Blame Three.js for forcing 1:1 canvas ratios... -->
        <div class="absolute top-[-4rem] left-0 h-[32rem] w-[16rem] min-w-[16rem] flex-shrink-0 p-0 m-0">
          <SkinPreviewRenderer
            slim-model-src="/src/assets/models/slim_player.gltf"
            wide-model-src="/src/assets/models/classic_player.gltf"
            :variant="variant"
            :texture-src="previewSkin"
            scale="1.4"
            fov="50"
            class="h-full w-full"
          />
        </div>
      </div>

      <div class="min-h-[16rem] flex flex-col gap-4 w-full">
        <section>
          <h2 class="text-base font-semibold mb-2">Texture</h2>
          <Card class="!bg-bg p-4 relative flex flex-col items-center gap-4">
            <FileInput
              :max-size="8000"
              accept="image/png"
              :prompt="mode === 'edit'? 'Replace skin' : 'Upload a skin'"
              class="btn btn-primary"
              :aria-label="mode === 'edit'? 'Replace skin' : 'Upload a skin'"
              @change="onTextureSelected"
            >
              <UploadIcon aria-hidden="true" />
            </FileInput>
              <div class="flex items-center gap-2 mx-auto">
                  <InfoIcon v-if="!fileName" aria-hidden="true" class="text-brand-blue" />
                  <CheckCircleIcon v-else aria-hidden="true" class="text-brand-green" />
                    <small class="max-w-64 truncate block" v-tooltip="fileName ?? currentSkin.texture ?? undefined">
                      {{ fileName || (mode === 'edit' ? currentSkin.texture : 'No skin has been uploaded yet.') }}
                    </small>
              </div>
          </Card>
        </section>
        <section>
          <h2 class="text-base font-semibold mb-2">Arm style</h2>
          <RadioButtons
            v-model="variant"
            :items="['Classic', 'Slim']">
            <template #default="{ item }">
              {{ item === 'Classic' ? 'Wide' : 'Slim' }}
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
            <CapeLikeTextButton tooltip="View more capes" v-if="capes?.length > 3">
              <template #icon>
                <ChevronRightIcon />
              </template>
              <span>More</span>
            </CapeLikeTextButton>
          </div>
        </section>
      </div>
    </div>
  </ModalWrapper>
</template>
