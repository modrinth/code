<script setup lang="ts">
import {UpdatedIcon, PlusIcon, ExcitedRinthbot} from '@modrinth/assets'
import { ButtonStyled, SkinPreviewRenderer, SkinButton, SkinLikeTextButton } from '@modrinth/ui'
import { ref, computed, useTemplateRef, watch, onMounted, onUnmounted } from 'vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import { handleError } from '@/store/notifications'
import {
  get_available_skins,
  get_available_capes,
  filterSavedSkins,
  filterDefaultSkins,
  equip_skin,
  set_default_cape,
} from '@/helpers/skins.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import type { Cape, Skin } from '@/helpers/skins.ts'
import { get_default_user, users } from '@/helpers/auth'
import type { RenderResult } from '@/helpers/rendering/batchSkinRenderer.ts'
import { generateSkinPreviews, map } from '@/helpers/rendering/batchSkinRenderer.ts'
import { WavingRinthbot } from '@modrinth/assets'

const editSkinModal = useTemplateRef('editSkinModal')
const selectCapeModal = useTemplateRef('selectCapeModal')

const settings = ref(await getSettings())
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])
const currentUser = ref(undefined)
const currentUserId = ref<string | undefined>(undefined)

// @ts-ignore
const username = computed(() => currentUser.value?.profile?.name ?? undefined)
const selectedSkin = ref<Skin | null>(null)
const defaultCape = ref<Cape>()

const savedSkins = computed(() => filterSavedSkins(skins.value))
const defaultSkins = computed(() => filterDefaultSkins(skins.value))

const currentCape = computed(() => {
  if (selectedSkin.value?.cape_id) {
    const overrideCape = capes.value.find((c) => c.id === selectedSkin.value?.cape_id)
    if (overrideCape) {
      return overrideCape
    }
  }
  return defaultCape.value
})

const skinTexture = computed(() => selectedSkin.value?.texture ?? '')
const capeTexture = computed(() => currentCape.value?.texture)
const skinVariant = computed(() => selectedSkin.value?.variant)
const skinNametag = computed(() => settings.value.hide_nametag_skins_page ? undefined : username.value)

await Promise.all([loadCapes(), loadSkins(), loadCurrentUser()])

let userCheckInterval: number | null = null

onMounted(() => {
  // TODO: Surely a better way to do this?
  userCheckInterval = window.setInterval(checkUserChanges, 250)
})

onUnmounted(() => {
  if (userCheckInterval !== null) {
    window.clearInterval(userCheckInterval)
  }
})

async function checkUserChanges() {
  try {
    const defaultId = await get_default_user()
    if (defaultId !== currentUserId.value) {
      await Promise.all([loadCapes(), loadSkins(), loadCurrentUser()])
    }
  } catch (e) {
    handleError(e)
  }
}

async function loadCapes() {
  capes.value = (await get_available_capes().catch(handleError)) ?? []
  defaultCape.value = capes.value.find((c) => c.is_equipped)
}

async function loadSkins() {
  skins.value = (await get_available_skins().catch(handleError)) ?? []
  generateSkinPreviews(skins.value, capes.value)
  selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null
}

async function changeSkin(newSkin: Skin) {
  await equip_skin(newSkin).catch(handleError)
  await loadSkins()
}

async function handleCapeSelected(cape: Cape | undefined) {
  await set_default_cape(cape).catch(handleError)
  await loadSkins()
  await loadCapes()
}

async function onSkinSaved() {
  await Promise.all([loadCapes(), loadSkins()])
}

async function loadCurrentUser() {
  try {
    const defaultId = await get_default_user()
    currentUserId.value = defaultId

    const allAccounts = await users()
    // @ts-ignore
    currentUser.value = allAccounts.find((acc) => acc.profile.id === defaultId)
  } catch (e) {
    handleError(e)
    currentUser.value = undefined
    currentUserId.value = undefined
  }
}

function getBakedSkinTextures(skin: Skin): RenderResult | undefined {
  const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`
  return map.get(key)
}

watch(
  () => selectedSkin.value?.cape_id,
  () => {}
)
</script>

<template>
  <EditSkinModal
    ref="editSkinModal"
    :capes="capes"
    @saved="onSkinSaved"
    @deleted="() => loadSkins()"
  />
  <SelectCapeModal ref="selectCapeModal" :capes="capes" @select="handleCapeSelected" />

  <div v-if="currentUser" class="p-6 grid grid-cols-[300px_1fr] xl:grid-cols-[3fr_5fr] gap-6">
    <div class="sticky top-6 self-start">
      <div class="flex justify-between gap-4">
        <h1 class="m-0 text-2xl font-bold">Skins</h1>
        <ButtonStyled :disabled="!!selectedSkin?.cape_id">
          <button
            v-tooltip="
      selectedSkin?.cape_id
        ? 'The equipped skin is overriding the default cape.'
        : undefined
        "
            :disabled="!!selectedSkin?.cape_id"
            @click="
          (e: MouseEvent) => selectCapeModal?.show(
            e,
            selectedSkin?.texture_key,
            currentCape,
            skinTexture,
            skinVariant
          )
        "
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
          cape-model-src="/src/assets/models/cape.gltf"
          :cape-src="capeTexture"
          :texture-src="skinTexture"
          :variant="skinVariant"
          :nametag="skinNametag"
          :initial-rotation="Math.PI / 8"
        />
      </div>
    </div>

    <div class="flex flex-col gap-6 add-perspective">
      <section class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Saved skins</h2>
        <div class="flex flex-row flex-wrap gap-2">
          <SkinLikeTextButton
            class="flex-none w-[8vw] h-[15vw] max-w-[8rem] max-h-[15rem]"
            tooltip="Add a skin"
            @click="editSkinModal?.show"
          >
            <template #icon>
              <PlusIcon />
            </template>
            Add a skin
          </SkinLikeTextButton>

          <SkinButton
            v-for="skin in savedSkins"
            :key="`saved-skin-${skin.texture_key}`"
            class="flex-none w-[8vw] h-[15vw] max-w-[8rem] max-h-[15rem]"
            editable
            :forward-image-src="getBakedSkinTextures(skin)?.forwards"
            :backward-image-src="getBakedSkinTextures(skin)?.backwards"
            :selected="selectedSkin === skin"
            @select="changeSkin(skin)"
            @edit="(e) => editSkinModal?.show(e, skin)"
          />
        </div>
      </section>

      <section class="flex flex-col gap-3">
        <h2 class="text-lg font-bold m-0 text-primary">Default skins</h2>
        <div class="flex flex-row flex-wrap gap-2">
          <SkinButton
            v-for="skin in defaultSkins"
            :key="`default-skin-${skin.texture_key}`"
            class="flex-none w-[8vw] h-[15vw] max-w-[8rem] max-h-[15rem]"
            :forward-image-src="getBakedSkinTextures(skin)?.forwards"
            :backward-image-src="getBakedSkinTextures(skin)?.backwards"
            :selected="selectedSkin === skin"
            :tooltip="skin.name"
            @select="changeSkin(skin)"
            @edit="(e) => editSkinModal?.show(e, skin)"
          />
        </div>
      </section>
    </div>
  </div>

  <div v-else class="flex items-center justify-center min-h-[50vh] p-6 pt-28">
    <div class="bg-bg-raised rounded-lg p-7 flex flex-col gap-5 shadow-md relative max-w-xl w-full mx-auto">
      <img :src="ExcitedRinthbot" alt="Excited Modrinth Bot" class="absolute -top-28 right-8 md:right-20 h-28 w-auto" />
      <div class="absolute top-0 left-0 w-full h-[1px] opacity-40 bg-gradient-to-r from-transparent via-green-500 to-transparent" style="background: linear-gradient(to right, transparent 2rem, var(--color-green) calc(100% - 13rem), var(--color-green) calc(100% - 5rem), transparent calc(100% - 2rem))"></div>

      <div class="flex flex-col gap-5">
        <h1 class="text-3xl font-extrabold m-0">Login Required</h1>
        <p class="text-lg m-0">
          Please log into your account to use the skin management features of the Modrinth app.
        </p>
      </div>
    </div>
  </div>
</template>
