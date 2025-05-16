<script setup lang="ts">
import { UpdatedIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, SkinPreviewRenderer, SkinButton, SkinLikeTextButton } from '@modrinth/ui'
import {ref, computed, useTemplateRef, onMounted} from 'vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import { handleError } from '@/store/notifications'
import {
  get_available_skins,
  get_available_capes,
  filterSavedSkins,
  filterDefaultSkins, equip_skin, remove_custom_skin, set_default_cape, add_and_equip_custom_skin,
} from '@/helpers/skins.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import { get as getCreds } from '@/helpers/mr_auth'
import { get_user } from '@/helpers/cache'
import type { Cape, Skin } from '@/helpers/skins.ts'
import {get_default_user, users} from "@/helpers/auth";
import {generateSkinPreviews, map} from "@/helpers/rendering/batchSkinRenderer.ts";

const editSkinModal = useTemplateRef('editSkinModal')
const selectCapeModal = useTemplateRef('selectCapeModal')

const settings = ref(await getSettings())
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])
const username = ref<string | undefined>(undefined)

const selectedSkin = ref<Skin | null>(null)

const previewSkin = computed(() =>
  selectedSkin.value ? `https://vzge.me/processedskin/${selectedSkin.value.texture_key}.png` : ''
)

const savedSkins = computed(() => filterSavedSkins(skins.value))
const defaultSkins = computed(() => filterDefaultSkins(skins.value))

const currentCape = ref<Cape>()

await Promise.all([loadCapes(), loadSkins(), loadUsername()])

async function loadCapes() {
  capes.value = (await get_available_capes().catch(handleError)) ?? []
  currentCape.value = capes.value.find((c) => c.is_equipped)
}

async function loadSkins() {
  skins.value = (await get_available_skins().catch(handleError)) ?? []
  generateSkinPreviews(skins.value);
  selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null;
}

async function changeSkin(newSkin: Skin) {
  await equip_skin(newSkin).catch(handleError);
  await loadSkins();
}

async function handleCapeSelected(cape: Cape | undefined) {
  currentCape.value = cape;
  await set_default_cape(currentCape.value).catch(handleError);
  await loadSkins();
  await loadCapes();
}

async function loadUsername() {
  try {
    const defaultId = await get_default_user()
    const allAccounts = await users();
    const current = allAccounts.find(acc => acc.profile.id === defaultId)
    username.value = current?.profile?.name ?? undefined
  } catch (e) {
    handleError(e)
    username.value = undefined
  }
}
</script>

<template>
  <EditSkinModal
    ref="editSkinModal"
    :capes="capes"
    @saved="() => loadSkins()"
    @deleted="() => loadSkins()"
  />
  <SelectCapeModal ref="selectCapeModal" :capes="capes" @select="handleCapeSelected"/>
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
          :nametag="settings.hide_nametag_skins_page ? undefined : username"
          :texture-src="previewSkin"
          :variant="selectedSkin?.variant"
        />
      </div>
    </div>

    <div class="flex flex-col gap-6 add-perspective">
      <section class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Saved skins</h2>
        <div class="flex flex-row flex-wrap gap-2">
          <SkinLikeTextButton @click="editSkinModal?.show" class="flex-none w-[8vw] h-[15vw] max-w-[8rem] max-h-[15rem]" tooltip="Add a skin">
            <template #icon>
              <PlusIcon />
            </template>
            Add a skin
          </SkinLikeTextButton>

          <SkinButton
            v-for="skin in savedSkins"
            class="flex-none w-[8vw] h-[15vw] max-w-[8rem] max-h-[15rem]"
            :key="`saved-skin-${skin.texture_key}`"
            editable
            :forward-image-src="map.get(skin.texture_key)?.forwards ?? ''"
            :backward-image-src="map.get(skin.texture_key)?.backwards ?? ''"
            :selected="selectedSkin === skin"
            @select="changeSkin(skin)"
            @edit="e => editSkinModal?.show(e, skin)"
          />
        </div>
      </section>

      <section class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Default skins</h2>
        <div class="flex flex-row flex-wrap gap-2">
          <SkinButton
            v-for="skin in defaultSkins"
            class="flex-none w-[8vw] h-[15vw] max-w-[8rem] max-h-[15rem]"
            :key="`default-skin-${skin.texture_key}`"
            :forward-image-src="map.get(skin.texture_key)?.forwards ?? ''"
            :backward-image-src="map.get(skin.texture_key)?.backwards ?? ''"
            :selected="selectedSkin === skin"
            :tooltip="skin.name"
            @select="changeSkin(skin)"
            @edit="e => editSkinModal?.show(e, skin)"
          />
        </div>
      </section>
    </div>
  </div>
</template>
