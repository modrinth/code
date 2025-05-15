<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import {
  Card,
  Chips,
  FileInput,
  SkinPreviewRenderer,
  CapeButton,
  CapeLikeTextButton,
  Button, RadioButtons, ButtonStyled
} from '@modrinth/ui'
import {
  add_and_equip_custom_skin,
  type Cape,
  determineModelType,
  equip_skin, remove_custom_skin,
  type Skin,
  type SkinModel
} from '@/helpers/skins.ts'
import {handleError} from '@/store/notifications'
import {computed, ref, useTemplateRef, watch} from 'vue'
import {CheckCircleIcon, InfoIcon, UploadIcon, ChevronRightIcon, CheckIcon, SaveIcon, XIcon, TrashIcon } from "@modrinth/assets";

const modal = useTemplateRef('modal')
const mode = ref<'new' | 'edit'>('new')
const currentSkin = ref<Skin | null>(null)
const textureBlob = ref<Uint8Array | null>(null)
const variant = ref<SkinModel>('CLASSIC')
const selectedCape = ref<Cape | undefined>(undefined)
const tempSkinUrl = ref<string>('/src/assets/skins/steve.png')

const props = defineProps<{ capes?: Cape[] }>()
const firstThreeCapes = computed(() => props.capes?.slice(0, 3) ?? [])

const localPreviewUrl = ref<string | null>(null)
const fileName = ref<string | null>(null)

watch(textureBlob, async (blob, prev) => {
  if (prev && localPreviewUrl.value) URL.revokeObjectURL(localPreviewUrl.value)
  localPreviewUrl.value = blob ? URL.createObjectURL(new Blob([blob])) : null

  if (blob && variant.value === 'UNKNOWN' && localPreviewUrl.value) {
    try {
      variant.value = await determineModelType(localPreviewUrl.value)
    } catch (err) {
      handleError(err)
    }
  }
})

const previewSkin = computed(() => {
  if (localPreviewUrl.value) return localPreviewUrl.value
  if (currentSkin.value) return currentSkin.value.texture
  if (mode.value === 'new') return tempSkinUrl.value
  return ''
})

const emit = defineEmits<{
  (e: 'saved', editedSkin: Skin | null, oldSkin: Skin): void
  (e: 'deleted', deletedSkin: Skin): void
}>()

const disableSave = computed(() => mode.value === 'new' && !textureBlob.value)

function resetState() {
  mode.value = 'new'
  currentSkin.value = null
  textureBlob.value = null
  fileName.value = null

  if (localPreviewUrl.value) {
    URL.revokeObjectURL(localPreviewUrl.value)
    localPreviewUrl.value = null
  }

  variant.value = 'CLASSIC'
  selectedCape.value = undefined
}

function show(e: MouseEvent, skin?: Skin) {
  mode.value = skin ? 'edit' : 'new'
  currentSkin.value = skin ?? null
  textureBlob.value = null

  if (!skin) {
    tempSkinUrl.value = Math.random() <= 0.05
      ? '/src/assets/skins/herobrine.png'
      : '/src/assets/skins/steve.png'
  }

  variant.value = skin?.variant ?? 'CLASSIC'
  selectedCape.value = skin?.cape_id ? props.capes?.find(c => c.id === skin.cape_id) : undefined
  modal.value?.show(e)
}

function hide() {
  modal.value?.hide()
  setTimeout(resetState, 100);
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
  // TODO: later
}

async function handleDelete() {
  try {
    await remove_custom_skin(currentSkin.value!)
    emit('deleted', currentSkin.value!)
    hide();
  } catch (err) {
    handleError(err);
  }
}

async function save() {
  try {
    let edited: Skin | null = null;
    if (mode.value === 'new') {
      if (!textureBlob.value) throw new Error('Please upload a skin texture first.')
      await add_and_equip_custom_skin(textureBlob.value, variant.value, selectedCape.value)
    } else if (currentSkin.value) {
      edited = {
        ...currentSkin.value,
        variant: variant.value,
        cape_id: selectedCape.value?.id,
        is_equipped: true
      }
      if (textureBlob.value) {
        await add_and_equip_custom_skin(textureBlob.value, variant.value, selectedCape.value)
      } else {
        await equip_skin(edited);
      }
    }
    emit('saved', edited, currentSkin.value!)
    hide()
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
            :scale="1.4"
            :fov="50"
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
              <div class="flex items-center gap-2 mx-auto" v-if="fileName || mode === 'new'">
                  <InfoIcon v-if="!fileName" aria-hidden="true" class="text-brand-blue" />
                  <CheckCircleIcon v-else aria-hidden="true" class="text-brand-green" />
                    <small class="max-w-64 truncate block" v-tooltip="fileName ?? undefined">
                      {{ fileName || 'No skin has been uploaded yet.' }}
                    </small>
              </div>
          </Card>
        </section>
        <section>
          <h2 class="text-base font-semibold mb-2">Arm style</h2>
          <RadioButtons
            v-model="variant"
            :items="['CLASSIC', 'SLIM']">
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
            <CapeLikeTextButton tooltip="View more capes" v-if="(capes?.length || 0) > 3">
              <template #icon>
                <ChevronRightIcon />
              </template>
              <span>More</span>
            </CapeLikeTextButton>
          </div>
        </section>
      </div>
    </div>
    <div class="flex flex-row gap-2 mt-6">
      <ButtonStyled color="brand" :disabled="disableSave">
        <Button @click="save" v-tooltip="disableSave && 'You need to upload a skin first!'" :disabled="disableSave">
          <CheckIcon v-if="mode === 'new'"/>
          <SaveIcon v-else />
          {{ (mode === 'edit' ? 'Save skin' : 'Add skin') }}
        </Button>
      </ButtonStyled>
      <Button @click="hide">
        <XIcon />
        Cancel
      </Button>
      <ButtonStyled color="red" v-if="mode === 'edit'">
        <Button @click="handleDelete">
          <TrashIcon />
          Delete
        </Button>
      </ButtonStyled>
    </div>
  </ModalWrapper>
</template>
