<template>
  <NewModal ref="modal" @on-hide="resetState">
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

      <div class="flex flex-col gap-4 w-full min-h-[20rem]">
        <section>
          <h2 class="text-base font-semibold mb-2">Texture</h2>
          <Button @click="openUploadSkinModal">
            <UploadIcon /> Replace texture
          </Button>
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
              <span>None</span>
            </CapeLikeTextButton>

            <CapeButton
              v-for="cape in visibleCapeList"
              :id="cape.id"
              :key="cape.id"
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

    <div class="flex gap-2 mt-12">
      <ButtonStyled color="brand" :disabled="disableSave">
        <Button v-tooltip="saveTooltip" :disabled="disableSave" @click="save">
          <CheckIcon v-if="mode === 'new'" /><SaveIcon v-else />
          {{ mode === 'new' ? 'Add skin' : 'Save skin' }}
        </Button>
      </ButtonStyled>
      <Button @click="hide"><XIcon/>Cancel</Button>
    </div>
  </NewModal>

  <SelectCapeModal ref="selectCapeModal" :capes="capes || []" @select="handleCapeSelected" @cancel="handleCapeCancel" />
</template>

<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import {
  Card, FileInput, SkinPreviewRenderer,
  Button, RadioButtons, CapeButton, CapeLikeTextButton, ButtonStyled,
  NewModal
} from '@modrinth/ui'
import {
  add_and_equip_custom_skin,
  remove_custom_skin,
  unequip_skin,
  type Skin, type Cape, type SkinModel
} from '@/helpers/skins.ts'
import { handleError } from '@/store/notifications'
import {
  UploadIcon,
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

const sortedCapes = computed(() => {
  return [...(props.capes || [])].sort((a, b) => {
    const nameA = (a.name || '').toLowerCase();
    const nameB = (b.name || '').toLowerCase();
    return nameA.localeCompare(nameB);
  });
});

function initVisibleCapeList() {
  if (!props.capes || props.capes.length === 0) {
    visibleCapeList.value = []
    return
  }

  if (visibleCapeList.value.length === 0) {
    if (selectedCape.value) {
      const otherCape = getSortedCapeExcluding(selectedCape.value.id)
      visibleCapeList.value = otherCape ? [selectedCape.value, otherCape] : [selectedCape.value]
    } else {
      visibleCapeList.value = getSortedCapes(2)
    }
  }
}

function getSortedCapes(count: number): Cape[] {
  if (!sortedCapes.value || sortedCapes.value.length === 0) return []
  return sortedCapes.value.slice(0, count)
}

function getSortedCapeExcluding(excludeId: string): Cape | undefined {
  if (!sortedCapes.value || sortedCapes.value.length <= 1) return undefined
  return sortedCapes.value.find(cape => cape.id !== excludeId)
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

const hasEdits = computed(() => {
  if (mode.value !== 'edit') return true;
  if (fileUploadTextureBlob.value) return true;
  if (!currentSkin.value) return false;
  if (variant.value !== currentSkin.value.variant) return true;
  if ((selectedCape.value?.id || null) !== (currentSkin.value.cape_id || null)) return true;
  return false;
});

const disableSave = computed(() =>
  (mode.value === 'new' && !fileUploadTextureBlob.value) ||
  (mode.value === 'edit' && !hasEdits.value)
);

const saveTooltip = computed(() => {
  if (mode.value === 'new' && !fileUploadTextureBlob.value) return 'Upload a skin first!';
  if (mode.value === 'edit' && !hasEdits.value) return 'Make an edit to the skin first!';
  return undefined;
});

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

function showNew(e: MouseEvent, skinTexture: Uint8Array, filename: string) {
  mode.value = 'new'
  currentSkin.value = null
  fileUploadTextureBlob.value = skinTexture
  fileName.value = filename
  variant.value = 'CLASSIC'
  selectedCape.value = undefined
  visibleCapeList.value = []
  initVisibleCapeList()
  modal.value?.show(e)
}

function restoreWithNewTexture(skinTexture: Uint8Array, filename: string) {
  fileUploadTextureBlob.value = skinTexture
  fileName.value = filename
  
  if (shouldRestoreModal.value) {
    setTimeout(() => {
      modal.value?.show()
      shouldRestoreModal.value = false
    }, 0)
  }
}

function hide() {
  modal.value?.hide()
  setTimeout(() => resetState(), 250)
}

function selectCape(cape: Cape|undefined) {
  if (cape && selectedCape.value?.id !== cape.id) {
    const isInVisibleList = visibleCapeList.value.some(c => c.id === cape.id)
    if (!isInVisibleList && visibleCapeList.value.length > 0) {
      visibleCapeList.value.splice(0, 1, cape)
    
      if (visibleCapeList.value.length > 1 && visibleCapeList.value[1].id === cape.id) {
        const otherCape = getSortedCapeExcluding(cape.id)
        if (otherCape) {
          visibleCapeList.value.splice(1, 1, otherCape)
        }
      }
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

function handleCapeCancel() {
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

function openUploadSkinModal(e: MouseEvent) {
  shouldRestoreModal.value = true
  modal.value?.hide()
  emit('open-upload-modal', e)
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
  (event: 'saved'): void
  (event: 'deleted', skin: Skin): void
  (event: 'open-upload-modal', mouseEvent: MouseEvent): void
}>()

defineExpose({
  show,
  showNew,
  restoreWithNewTexture,
  hide,
  shouldRestoreModal
})
</script>
