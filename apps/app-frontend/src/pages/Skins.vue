<script setup lang="ts">
import { UpdatedIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, SkinPreviewRenderer } from '@modrinth/ui'
import { ref, computed, useTemplateRef } from 'vue'
import SkinButton from '@/components/ui/skin/SkinButton.vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import { handleError } from '@/store/notifications'
import {
  get_available_skins,
  get_available_capes,
  filterSavedSkins,
  filterDefaultSkins, equip_skin, remove_custom_skin,
} from '@/helpers/skins.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import { get as getCreds } from '@/helpers/mr_auth'
import { get_user } from '@/helpers/cache'
import type { Cape, Skin } from '@/helpers/skins.ts'

const editSkinModal = useTemplateRef('editSkinModal')
const selectCapeModal = useTemplateRef('selectCapeModal')

const settings = ref(await getSettings())
const credentials = ref()
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])

const selectedSkin = ref<Skin | null>(null)

const previewSkin = computed(() =>
  selectedSkin.value ? `https://vzge.me/processedskin/${selectedSkin.value.texture_key}.png` : ''
)

const savedSkins = computed(() => filterSavedSkins(skins.value))
const defaultSkins = computed(() => filterDefaultSkins(skins.value))

const currentCape = ref<Cape>()

await Promise.all([fetchCredentials(), loadCapes(), loadSkins()])

async function fetchCredentials() {
  const creds = await getCreds().catch(handleError)
  if (creds?.user_id) {
    creds.user = await get_user(creds.user_id).catch(handleError)
  }
  credentials.value = creds
}

async function loadCapes() {
  capes.value = (await get_available_capes().catch(handleError)) ?? []
  currentCape.value = capes.value.find((c) => c.is_equipped)
}

async function loadSkins() {
  skins.value = (await get_available_skins().catch(handleError)) ?? []
  selectedSkin.value =
    skins.value.find((s) => s.texture_key === 'its_imb11') ?? skins.value[0] ?? null
}

async function changeSkin(newSkin: Skin) {
  selectedSkin.value = newSkin;
  await equip_skin(selectedSkin.value).catch(handleError);
}

async function handleSkinSaved(newSkin: Skin | null, oldSkin: Skin | null) {
  if (oldSkin) {
    await remove_custom_skin(oldSkin).catch(handleError)
    skins.value = skins.value.filter(s => s.texture_key !== oldSkin.texture_key)
  }

  if (newSkin) skins.value.push(newSkin)
  selectedSkin.value =
    skins.value.find(s => s.texture_key === newSkin?.texture_key) ??
    skins.value.find(s => s.is_equipped) ??
    skins.value[0] ??
    null
}

async function handleSkinDeleted(deletedSkin: Skin) {
  skins.value = skins.value.filter(s => s.texture_key !== deletedSkin.texture_key)
  if (selectedSkin.value?.texture_key === deletedSkin.texture_key) {
    selectedSkin.value =
      skins.value.find(s => s.is_equipped) ??
      skins.value[0] ??
      null
  }
}
</script>

<template>
  <EditSkinModal
    ref="editSkinModal"
    :capes="capes"
    @saved="handleSkinSaved"
    @deleted="handleSkinDeleted"
  />
  <SelectCapeModal ref="selectCapeModal" :capes="capes"/>
  <div class="p-6 grid grid-cols-[300px_1fr] xl:grid-cols-[3fr_5fr] gap-6">
    <div class="sticky top-6 self-start">
      <div class="flex justify-between gap-4">
        <h1 class="m-0 text-2xl font-bold">Skins</h1>
        <ButtonStyled>
          <button
            @click="(e: MouseEvent) =>
              selectCapeModal?.show(e, selectedSkin?.texture_key, currentCape)"
          >
            <UpdatedIcon />
            Change cape
          </button>
        </ButtonStyled>
      </div>
      <div class="h-[80vh] flex items-center justify-center">
        <SkinPreviewRenderer
          wide-model-src="/src/assets/models/classic_player.gltf"
          slim-model-src="/src/assets/models/slim_player.gltf"
          :nametag="settings.hide_nametag_skins_page ? undefined : credentials?.user?.username"
          :texture-src="previewSkin"
        />
      </div>
    </div>

    <div class="flex flex-col gap-6 add-perspective">
      <section class="flex flex-col gap-3">
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
            :selected="selectedSkin === skin"
            @select="changeSkin(skin)"
            @edit="e => editSkinModal?.show(e, skin)"
          />
        </div>
      </section>

      <section class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Default skins</h2>
        <div class="grid grid-cols-3 gap-2">
          <SkinButton
            v-for="skin in defaultSkins"
            :key="`default-skin-${skin.texture_key}`"
            :skin="skin"
            :selected="selectedSkin === skin"
            @select="changeSkin(skin)"
            @edit="e => editSkinModal?.show(e, skin)"
          />
        </div>
      </section>
    </div>
  </div>
</template>
