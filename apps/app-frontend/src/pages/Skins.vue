<script setup lang="ts">
import { UpdatedIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, SkinPreviewRenderer } from '@modrinth/ui'
import { ref, computed, useTemplateRef } from 'vue'
import SkinButton from '@/components/ui/skin/SkinButton.vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import type { Cape, Skin, SkinModel } from '@/helpers/skins.ts'
import { get_available_skins, get_available_capes } from '@/helpers/skins.ts'
import { handleError } from '@/store/notifications'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import { get as getSettings } from "@/helpers/settings.ts";

const editSkinModal = useTemplateRef('editSkinModal')
const selectCapeModal = useTemplateRef('selectCapeModal')

const settings = ref(await getSettings());

const selectedSkin = ref('its_imb11')
const previewSkin = computed(() => `https://vzge.me/processedskin/${selectedSkin.value}.png`)

const savedSkins = computed(() => skins.value.filter((skin) => skin.source !== 'Default'))
const defaultSkins = computed(() =>
  skins.value
    .filter(
      (skin) =>
        skin.source === 'Default' && (!skin.name || skin.variant === defaultModels[skin.name]),
    )
    .sort((a, b) => {
      if (!a.name || !defaultModelSorting.includes(a.name)) {
        return 1
      } else if (!b.name || !defaultModelSorting.includes(b.name)) {
        return -1
      }

      return defaultModelSorting.indexOf(a.name) - defaultModelSorting.indexOf(b.name)
    }),
)
const currentCape = ref<Cape | undefined>()

const defaultModelSorting = ['Steve', 'Alex']

const defaultModels: Record<string, SkinModel> = {
  Steve: 'Classic',
  Alex: 'Slim',
  Zuri: 'Classic',
  Sunny: 'Classic',
  Noor: 'Slim',
  Makena: 'Slim',
  Kai: 'Classic',
  Efe: 'Slim',
  Ari: 'Classic',
}

const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])

await loadCapes()
await loadSkins()

async function loadCapes() {
  await get_available_capes()
    .then((c) => {
      capes.value = c
      currentCape.value = capes.value.find((cape) => cape.is_equipped)
      console.log(c)
    })
    .catch((err) => handleError(err))
}

async function loadSkins() {
  await get_available_skins()
    .then((s) => {
      skins.value = s
      console.log(s)
    })
    .catch((err) => handleError(err))
}
</script>
<template>
  <EditSkinModal ref="editSkinModal" />
  <SelectCapeModal ref="selectCapeModal" :capes="capes" />
  <div class="p-6 grid grid-cols-[300px_1fr] xl:grid-cols-[3fr_5fr] gap-6">
    <div class="sticky top-6 self-start">
      <div class="flex justify-between gap-4">
        <h1 class="m-0 text-2xl font-extrabold">Skins</h1>
        <div>
          <ButtonStyled>
            <button @click="(e: MouseEvent) => selectCapeModal?.show(e, selectedSkin, currentCape)">
              <UpdatedIcon />
              Change cape
            </button>
          </ButtonStyled>
        </div>
      </div>
      <div class="h-[80vh] flex items-center justify-center">
        <SkinPreviewRenderer :model-src="'/src/assets/models/wide_player.gltf'" :nametag="settings.hide_nametag_skins_page ? undefined : selectedSkin" :texture-src="previewSkin" />
      </div>
    </div>
    <div class="flex flex-col gap-6 add-perspective">
      <div class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Saved skins</h2>
        <div class="grid grid-cols-3 gap-2">
          <button
            class="flex flex-col gap-3 active:scale-95 hover:brightness-125 font-medium text-primary items-center justify-center border-2 border-transparent border-solid cursor-pointer h-40 bg-button-bg rounded-xl"
            @click="editSkinModal?.show"
          >
            <PlusIcon class="w-6 h-6" />
            Add a skin
          </button>
          <SkinButton
            v-for="skin in savedSkins"
            :key="`saved-skin-${skin.texture_key}`"
            editable
            :skin="skin"
            :selected="selectedSkin === skin.texture_key"
            @select="selectedSkin = skin.texture_key"
            @edit="editSkinModal?.show"
          />
        </div>
      </div>
      <div class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Default skins</h2>
        <div class="grid grid-cols-3 gap-2">
          <SkinButton
            v-for="skin in defaultSkins"
            :key="`default-skin-${skin.texture_key}`"
            :skin="skin"
            :selected="selectedSkin === skin.texture_key"
            @select="selectedSkin = skin.texture_key"
            @edit="editSkinModal?.show"
          />
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped></style>
