<script setup lang="ts">
import {
  UpdatedIcon,
  PlusIcon,
  ExcitedRinthbot,
  LogInIcon,
  SpinnerIcon,
  EditIcon,
  TrashIcon,
} from '@modrinth/assets'
import {
  ButtonStyled,
  SkinPreviewRenderer,
  SkinButton,
  SkinLikeTextButton,
  Button,
  ConfirmModal,
} from '@modrinth/ui'
import type { Ref } from 'vue'
import { ref, computed, useTemplateRef, watch, onMounted, onUnmounted, inject } from 'vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import UploadSkinModal from '@/components/ui/skin/UploadSkinModal.vue'
import { handleError } from '@/store/notifications'
import {
  get_available_skins,
  get_available_capes,
  filterSavedSkins,
  filterDefaultSkins,
  equip_skin,
  set_default_cape,
  remove_custom_skin,
} from '@/helpers/skins.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import type { Cape, Skin } from '@/helpers/skins.ts'
import { get_default_user, users, login as login_flow } from '@/helpers/auth'
import type { RenderResult } from '@/helpers/rendering/batch-skin-renderer.ts'
import { generateSkinPreviews, map } from '@/helpers/rendering/batch-skin-renderer.ts'
import { handleSevereError } from '@/store/error'
import { trackEvent } from '@/helpers/analytics'
import type AccountsCard from '@/components/ui/AccountsCard.vue'

const editSkinModal = useTemplateRef('editSkinModal')
const selectCapeModal = useTemplateRef('selectCapeModal')
const uploadSkinModal = useTemplateRef('uploadSkinModal')

const settings = ref(await getSettings())
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])

const accountsCard = inject('accountsCard') as Ref<typeof AccountsCard>
const currentUser = ref(undefined)
const currentUserId = ref<string | undefined>(undefined)

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
const skinNametag = computed(() =>
  settings.value.hide_nametag_skins_page ? undefined : username.value,
)

let userCheckInterval: number | null = null

const deleteSkinModal = ref()
const skinToDelete = ref<Skin | null>(null)

function confirmDeleteSkin(skin: Skin) {
  skinToDelete.value = skin
  deleteSkinModal.value?.show()
}

async function deleteSkin() {
  if (!skinToDelete.value) return
  await remove_custom_skin(skinToDelete.value).catch(handleError)
  await loadSkins()
  skinToDelete.value = null
}

async function loadCapes() {
  try {
    capes.value = (await get_available_capes()) ?? []
    defaultCape.value = capes.value.find((c) => c.is_equipped)
  } catch (error) {
    if (currentUser.value) {
      handleError(error)
    }
  }
}

async function loadSkins() {
  try {
    skins.value = (await get_available_skins()) ?? []
    generateSkinPreviews(skins.value, capes.value)
    selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null
  } catch (error) {
    if (currentUser.value) {
      handleError(error)
    }
  }
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

async function login() {
  accountsCard.value.setLoginDisabled(true)
  const loggedIn = await login_flow().catch(handleSevereError)

  if (loggedIn && accountsCard) {
    await accountsCard.value.refreshValues()
  }

  trackEvent('AccountLogIn')
  accountsCard.value.setLoginDisabled(false)
}

function openUploadSkinModal(e: MouseEvent) {
  uploadSkinModal.value?.show(e)
}

function onSkinFileUploaded(file: File) {
  const fakeEvent = new MouseEvent('click')
  file.arrayBuffer().then((buf) => {
    const skinTexture = new Uint8Array(buf)

    if (editSkinModal.value && editSkinModal.value.shouldRestoreModal) {
      editSkinModal.value.restoreWithNewTexture(skinTexture, file.name)
    } else {
      editSkinModal.value?.showNew(fakeEvent, skinTexture, file.name)
    }
  })
}

function onUploadCanceled() {
  if (editSkinModal.value && editSkinModal.value.shouldRestoreModal) {
    setTimeout(() => {
      const fakeEvent = new MouseEvent('click')
      editSkinModal.value!.show(fakeEvent)
      editSkinModal.value!.shouldRestoreModal = false
    }, 0)
  }
}

watch(
  () => selectedSkin.value?.cape_id,
  () => {},
)

onMounted(() => {
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
      await loadCurrentUser()
      await loadCapes()
      await loadSkins()
    }
  } catch (error) {
    if (currentUser.value) {
      handleError(error)
    }
  }
}

await Promise.all([loadCapes(), loadSkins(), loadCurrentUser()])
</script>

<template>
  <EditSkinModal
    ref="editSkinModal"
    :capes="capes"
    @saved="onSkinSaved"
    @deleted="() => loadSkins()"
    @open-upload-modal="openUploadSkinModal"
  />
  <SelectCapeModal ref="selectCapeModal" :capes="capes" @select="handleCapeSelected" />
  <UploadSkinModal
    ref="uploadSkinModal"
    @uploaded="onSkinFileUploaded"
    @canceled="onUploadCanceled"
  />
  <ConfirmModal
    ref="deleteSkinModal"
    title="Are you sure you want to delete this skin?"
    description="This will permanently delete the selected skin. This action cannot be undone."
    proceed-label="Delete"
    @proceed="deleteSkin"
  />

  <div v-if="currentUser" class="p-4 skin-layout">
    <div class="preview-panel">
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
              (e: MouseEvent) =>
                selectCapeModal?.show(
                  e,
                  selectedSkin?.texture_key,
                  currentCape,
                  skinTexture,
                  skinVariant,
                )
            "
          >
            <UpdatedIcon />
            Change cape
          </button>
        </ButtonStyled>
      </div>
      <div class="preview-container">
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

    <div class="skins-container">
      <section class="flex flex-col gap-2 mt-1">
        <h2 class="text-lg font-bold m-0 text-primary">Saved skins</h2>
        <div class="skin-card-grid">
          <SkinLikeTextButton class="skin-card" tooltip="Add a skin" @click="openUploadSkinModal">
            <template #icon>
              <PlusIcon class="size-8" />
            </template>
            <span>Add a skin</span>
          </SkinLikeTextButton>

          <SkinButton
            v-for="skin in savedSkins"
            :key="`saved-skin-${skin.texture_key}`"
            class="skin-card"
            :forward-image-src="getBakedSkinTextures(skin)?.forwards"
            :backward-image-src="getBakedSkinTextures(skin)?.backwards"
            :selected="selectedSkin === skin"
            @select="changeSkin(skin)"
          >
            <template #overlay-buttons>
              <Button
                color="green"
                aria-label="Edit skin"
                @click.stop="(e) => editSkinModal?.show(e, skin)"
              >
                <EditIcon /> Edit
              </Button>
              <Button
                v-show="!skin.is_equipped"
                v-tooltip="'Delete skin'"
                aria-label="Delete skin"
                color="red"
                class="!rounded-[100%]"
                icon-only
                @click.stop="() => confirmDeleteSkin(skin)"
              >
                <TrashIcon />
              </Button>
            </template>
          </SkinButton>
        </div>
      </section>

      <section class="flex flex-col gap-2 mt-6">
        <h2 class="text-lg font-bold m-0 text-primary">Default skins</h2>
        <div class="skin-card-grid">
          <SkinButton
            v-for="skin in defaultSkins"
            :key="`default-skin-${skin.texture_key}`"
            class="skin-card"
            :forward-image-src="getBakedSkinTextures(skin)?.forwards"
            :backward-image-src="getBakedSkinTextures(skin)?.backwards"
            :selected="selectedSkin === skin"
            :tooltip="skin.name"
            @select="changeSkin(skin)"
          />
        </div>
      </section>
    </div>
  </div>

  <div v-else class="flex items-center justify-center min-h-[50vh] pt-[25%]">
    <div
      class="bg-bg-raised rounded-lg p-7 flex flex-col gap-5 shadow-md relative max-w-xl w-full mx-auto"
    >
      <img
        :src="ExcitedRinthbot"
        alt="Excited Modrinth Bot"
        class="absolute -top-28 right-8 md:right-20 h-28 w-auto"
      />
      <div
        class="absolute top-0 left-0 w-full h-[1px] opacity-40 bg-gradient-to-r from-transparent via-green-500 to-transparent"
        style="
          background: linear-gradient(
            to right,
            transparent 2rem,
            var(--color-green) calc(100% - 13rem),
            var(--color-green) calc(100% - 5rem),
            transparent calc(100% - 2rem)
          );
        "
      ></div>

      <div class="flex flex-col gap-5">
        <h1 class="text-3xl font-extrabold m-0">Please sign-in</h1>
        <p class="text-lg m-0">
          Please sign into your Minecraft account to use the skin management features of the
          Modrinth app.
        </p>
        <ButtonStyled v-show="accountsCard" color="brand" :disabled="accountsCard.loginDisabled">
          <Button :disabled="accountsCard.loginDisabled" @click="login">
            <LogInIcon v-if="!accountsCard.loginDisabled" />
            <SpinnerIcon v-else class="animate-spin" />
            Sign In
          </Button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
$skin-card-width: 155px;
$skin-card-gap: 4px;

.skin-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);
  gap: 2.5rem;

  @media (max-width: 700px) {
    grid-template-columns: 1fr;
  }
}

.preview-panel {
  position: sticky;
  top: 1.5rem;
  align-self: start;
  padding: 0.5rem;

  @media (max-width: 700px) {
    position: static;
  }
}

.preview-container {
  height: 80vh;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: calc((2.5rem / 2));

  @media (max-width: 700px) {
    height: 50vh;
  }
}

.skins-container {
  padding-top: 0.5rem;
}

.skin-card-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: $skin-card-gap;
  width: 100%;

  @media (max-width: 1110px) {
    grid-template-columns: repeat(3, 1fr);
  }
}

.skin-card {
  aspect-ratio: 0.95;
  border-radius: 10px;
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
}
</style>
