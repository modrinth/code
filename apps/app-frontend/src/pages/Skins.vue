<script setup lang="ts">
import {
	CheckIcon,
	EditIcon,
	ExcitedRinthbot,
	EyeIcon,
	LogInIcon,
	RotateCounterClockwiseIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	injectNotificationManager,
	SkinPreviewRenderer,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { type DragDropEvent, getCurrentWebview } from '@tauri-apps/api/webview'
import { computedAsync } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, inject, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'

import type AccountsCard from '@/components/ui/AccountsCard.vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import VirtualSkinSectionList from '@/components/ui/skin/VirtualSkinSectionList.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_default_user, login as login_flow, users } from '@/helpers/auth'
import type { RenderResult } from '@/helpers/rendering/batch-skin-renderer.ts'
import { generateSkinPreviews, skinBlobUrlMap } from '@/helpers/rendering/batch-skin-renderer.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import type { Cape, Skin, SkinTextureUrl } from '@/helpers/skins.ts'
import {
	equip_skin,
	filterDefaultSkins,
	filterSavedSkins,
	get_available_capes,
	get_available_skins,
	get_dragged_skin_data,
	get_normalized_skin_texture,
	normalize_skin_texture,
	remove_custom_skin,
} from '@/helpers/skins.ts'
import { handleSevereError } from '@/store/error'

type UnlistenFn = () => void
type VirtualSkinSectionListExpose = {
	getAddSkinButtonElement: () => HTMLElement | null | undefined
}

const editSkinModal = useTemplateRef('editSkinModal')
const addSkinFileInput = useTemplateRef<HTMLInputElement>('addSkinFileInput')
const skinSectionList = useTemplateRef<VirtualSkinSectionListExpose>('skinSectionList')

const notifications = injectNotificationManager()
const { addNotification, handleError } = notifications

const settings = ref(await getSettings())
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])

const accountsCard = inject('accountsCard') as Ref<typeof AccountsCard>
const currentUser = ref(undefined)
const currentUserId = ref<string | undefined>(undefined)

const username = computed(() => currentUser.value?.profile?.name ?? undefined)
const selectedSkin = ref<Skin | null>(null)
const isApplyingSkin = ref(false)

const originalSelectedSkin = ref<Skin | null>(null)

const savedSkins = computed(() => {
	try {
		return filterSavedSkins(skins.value)
	} catch (error) {
		handleError(error as Error)
		return []
	}
})
const defaultSkins = computed(() => filterDefaultSkins(skins.value))
const defaultSkinSections = computed(() => {
	const sections = new Map<string, Skin[]>()

	for (const skin of defaultSkins.value) {
		const sectionTitle = skin.section ?? 'Default skins'
		const sectionSkins = sections.get(sectionTitle)

		if (sectionSkins) {
			sectionSkins.push(skin)
		} else {
			sections.set(sectionTitle, [skin])
		}
	}

	return Array.from(sections, ([title, skins]) => ({ title, skins }))
})

const currentCape = computed(() => {
	if (selectedSkin.value?.cape_id) {
		const overrideCape = capes.value.find((c) => c.id === selectedSkin.value?.cape_id)
		if (overrideCape) {
			return overrideCape
		}
	}
	return undefined
})

const skinTexture = computedAsync(async () => {
	if (selectedSkin.value?.texture) {
		return await get_normalized_skin_texture(selectedSkin.value)
	} else {
		return ''
	}
})
const capeTexture = computed(() => currentCape.value?.texture)
const skinVariant = computed(() => selectedSkin.value?.variant)
const skinNametag = computed(() =>
	settings.value.hide_nametag_skins_page ? undefined : username.value,
)
const hasPendingSkinChange = computed(
	() => !skinsMatch(selectedSkin.value, originalSelectedSkin.value),
)

let userCheckInterval: number | null = null
let unlistenAddSkinDragDrop: UnlistenFn | null = null
let isUnmounted = false

const isDraggingSkinFile = ref(false)
const isAddSkinButtonDragActive = ref(false)

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
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

async function loadSkins() {
	try {
		skins.value = (await get_available_skins()) ?? []
		generateSkinPreviews(skins.value, capes.value)
		selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null
		originalSelectedSkin.value = selectedSkin.value
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

function skinsMatch(a?: Skin | null, b?: Skin | null) {
	return (
		a?.source === b?.source &&
		a?.texture_key === b?.texture_key &&
		a?.variant === b?.variant &&
		(a?.cape_id ?? null) === (b?.cape_id ?? null)
	)
}

function isSkinSelected(skin: Skin) {
	return skinsMatch(selectedSkin.value, skin)
}

function changeSkin(newSkin: Skin) {
	selectedSkin.value = newSkin
}

function resetSelectedSkin() {
	selectedSkin.value =
		skins.value.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
		originalSelectedSkin.value
}

async function applySelectedSkin() {
	const skinToApply = selectedSkin.value
	if (!skinToApply || !hasPendingSkinChange.value || isApplyingSkin.value) return

	isApplyingSkin.value = true
	try {
		await equip_skin(skinToApply)
		if (accountsCard.value) {
			await accountsCard.value.refreshValues()
		}
		await loadCapes()
		await loadSkins()
	} catch (error) {
		if ((error as { message?: string })?.message?.includes('429 Too Many Requests')) {
			notifications.addNotification({
				type: 'error',
				title: 'Slow down!',
				text: "You're changing your skin too frequently. Mojang's servers have temporarily blocked further requests. Please wait a moment before trying again.",
			})
		} else {
			handleError(error as Error)
		}
	} finally {
		isApplyingSkin.value = false
	}
}

async function onSkinSaved() {
	await loadCapes()
	await loadSkins()
}

async function loadCurrentUser() {
	try {
		const defaultId = await get_default_user()
		currentUserId.value = defaultId

		const allAccounts = await users()
		currentUser.value = allAccounts.find((acc) => acc.profile.id === defaultId)
	} catch (e) {
		handleError(e as Error)
		currentUser.value = undefined
		currentUserId.value = undefined
	}
}

function getBakedSkinTextures(skin: Skin): RenderResult | undefined {
	const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`
	return skinBlobUrlMap.get(key)
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

function openAddSkinFileBrowser() {
	addSkinFileInput.value?.click()
}

async function onAddSkinFileInputChange(e: Event) {
	const files = (e.target as HTMLInputElement).files
	const file = files?.[0]

	if (!file) {
		return
	}

	await processSkinFileBuffer(await file.arrayBuffer())

	if (addSkinFileInput.value) {
		addSkinFileInput.value.value = ''
	}
}

function isSkinImagePath(path: string) {
	return path.toLowerCase().endsWith('.png')
}

function isSkinFileDrag(event: DragEvent) {
	const items = Array.from(event.dataTransfer?.items ?? [])
	const files = Array.from(event.dataTransfer?.files ?? [])

	return (
		items.some((item) => item.kind === 'file' && item.type === 'image/png') ||
		files.some((file) => file.type === 'image/png' || isSkinImagePath(file.name))
	)
}

function isPositionOverAddSkinButton(position: { x: number; y: number }) {
	const element = skinSectionList.value?.getAddSkinButtonElement()

	if (!element) {
		return false
	}

	const { x, y } = position
	const rect = element.getBoundingClientRect()

	return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom
}

async function handleAddSkinNativeDragDrop(event: { payload: DragDropEvent }) {
	const payload = event.payload

	if (payload.type === 'leave') {
		isDraggingSkinFile.value = false
		isAddSkinButtonDragActive.value = false
		return
	}

	if (payload.type === 'enter') {
		isDraggingSkinFile.value = payload.paths.some(isSkinImagePath)
	}

	if (payload.type === 'enter' || payload.type === 'over') {
		isAddSkinButtonDragActive.value =
			isDraggingSkinFile.value && isPositionOverAddSkinButton(payload.position)
		return
	}

	const hasSkinPath = payload.paths.some(isSkinImagePath)
	const shouldUpload =
		(isDraggingSkinFile.value || hasSkinPath) && isPositionOverAddSkinButton(payload.position)

	isDraggingSkinFile.value = false
	isAddSkinButtonDragActive.value = false

	if (!shouldUpload) {
		return
	}

	const skinPath = payload.paths.find(isSkinImagePath)

	if (!skinPath) {
		return
	}

	try {
		const data = await get_dragged_skin_data(skinPath)
		await processSkinFileBuffer(data)
	} catch (error) {
		addNotification({
			title: 'Error processing file',
			text: error instanceof Error ? error.message : 'Failed to read the dropped file.',
			type: 'error',
		})
	}
}

function onAddSkinDragOver(event: DragEvent) {
	if (!isSkinFileDrag(event)) {
		return
	}

	isAddSkinButtonDragActive.value = true
}

function onAddSkinDragLeave() {
	isAddSkinButtonDragActive.value = false
}

async function onAddSkinDrop(event: DragEvent) {
	isAddSkinButtonDragActive.value = false

	const file = Array.from(event.dataTransfer?.files ?? []).find(
		(file) => file.type === 'image/png' || isSkinImagePath(file.name),
	)

	if (!file) {
		return
	}

	await processSkinFileBuffer(await file.arrayBuffer())
}

async function setupAddSkinDragDropListener() {
	try {
		const unlisten = await getCurrentWebview().onDragDropEvent(handleAddSkinNativeDragDrop)

		if (isUnmounted) {
			unlisten()
			return
		}

		unlistenAddSkinDragDrop = unlisten
	} catch (error) {
		handleError(error as Error)
	}
}

async function processSkinFileBuffer(buffer: Uint8Array | ArrayBuffer) {
	const fakeEvent = new MouseEvent('click')
	const originalSkinTexUrl = `data:image/png;base64,` + arrayBufferToBase64(buffer)
	try {
		const skinTextureNormalized = await normalize_skin_texture(originalSkinTexUrl)
		const skinTexUrl: SkinTextureUrl = {
			original: originalSkinTexUrl,
			normalized: `data:image/png;base64,` + arrayBufferToBase64(skinTextureNormalized),
		}
		editSkinModal.value?.showNew(fakeEvent, skinTexUrl)
	} catch (error) {
		handleError(error as Error)
	}
}

watch(
	() => selectedSkin.value?.cape_id,
	() => {},
)

onMounted(() => {
	userCheckInterval = window.setInterval(checkUserChanges, 250)
	void setupAddSkinDragDropListener()
})

onUnmounted(() => {
	isUnmounted = true

	if (userCheckInterval !== null) {
		window.clearInterval(userCheckInterval)
	}

	if (unlistenAddSkinDragDrop) {
		unlistenAddSkinDragDrop()
		unlistenAddSkinDragDrop = null
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
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

await Promise.all([loadCapes(), loadCurrentUser()])
await loadSkins()
</script>

<template>
	<EditSkinModal
		ref="editSkinModal"
		:capes="capes"
		@saved="onSkinSaved"
		@deleted="() => loadSkins()"
	/>
	<input
		ref="addSkinFileInput"
		type="file"
		accept="image/png"
		class="hidden"
		@change="onAddSkinFileInputChange"
	/>
	<ConfirmModal
		ref="deleteSkinModal"
		title="Are you sure you want to delete this skin?"
		description="This will permanently delete the selected skin. This action cannot be undone."
		proceed-label="Delete"
		@proceed="deleteSkin"
	/>

	<div v-if="currentUser" class="skin-layout box-border min-h-full p-4">
		<div class="sticky top-6 self-start p-2 pt-0">
			<h1 class="m-0 text-2xl font-bold flex items-center gap-2">Skins</h1>
			<div class="ml-5 flex h-[80vh] items-center justify-center max-[700px]:h-[50vh]">
				<SkinPreviewRenderer
					:cape-src="capeTexture"
					:texture-src="skinTexture || ''"
					:variant="skinVariant"
					:nametag="skinNametag"
					:initial-rotation="Math.PI / 8"
					:lock-fit="false"
				>
					<template v-if="hasPendingSkinChange" #nametag-badge>
						<div
							class="flex items-center justify-center gap-1.5 rounded-full border border-solid border-brand-blue bg-bg-blue px-3 py-1 text-base font-semibold leading-6 text-brand-blue"
						>
							<EyeIcon class="size-5 shrink-0" />
							Previewing
						</div>
					</template>
					<template #subtitle>
						<div
							v-if="hasPendingSkinChange"
							class="flex max-w-[calc(100vw-2rem)] flex-wrap items-center justify-center gap-2 px-2"
						>
							<button
								class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-2 rounded-[14px] border-0 bg-surface-4 px-4 py-2.5 text-base font-semibold leading-5 text-contrast shadow-md transition-[filter,transform] duration-200 enabled:hover:brightness-[--hover-brightness] enabled:focus-visible:brightness-[--hover-brightness] enabled:active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-5 [&>svg]:shrink-0"
								:disabled="isApplyingSkin"
								@click="resetSelectedSkin"
							>
								<RotateCounterClockwiseIcon />
								Reset
							</button>
							<button
								class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-2 rounded-[14px] border-0 bg-brand px-4 py-2.5 text-base font-semibold leading-5 text-[rgba(0,0,0,0.9)] shadow-md transition-[filter,transform] duration-200 enabled:hover:brightness-[--hover-brightness] enabled:focus-visible:brightness-[--hover-brightness] enabled:active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-5 [&>svg]:shrink-0"
								:disabled="isApplyingSkin"
								@click="applySelectedSkin"
							>
								<SpinnerIcon v-if="isApplyingSkin" class="animate-spin" />
								<CheckIcon v-else />
								Apply
							</button>
						</div>
						<button
							v-else
							class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-2 rounded-[14px] border-0 bg-surface-4 px-4 py-2.5 text-base font-semibold leading-5 text-contrast shadow-md transition-[filter,transform] duration-200 enabled:hover:brightness-[--hover-brightness] enabled:focus-visible:brightness-[--hover-brightness] enabled:active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-5 [&>svg]:shrink-0"
							:disabled="!selectedSkin"
							@click="(e: MouseEvent) => selectedSkin && editSkinModal?.show(e, selectedSkin)"
						>
							<EditIcon />
							Edit skin
						</button>
					</template>
				</SkinPreviewRenderer>
			</div>
		</div>

		<div class="pt-2">
			<VirtualSkinSectionList
				ref="skinSectionList"
				:saved-skins="savedSkins"
				:default-skin-sections="defaultSkinSections"
				:get-baked-skin-textures="getBakedSkinTextures"
				:is-skin-selected="isSkinSelected"
				:is-add-skin-button-drag-active="isAddSkinButtonDragActive"
				@select="changeSkin"
				@edit="(skin, event) => editSkinModal?.show(event, skin)"
				@delete="confirmDeleteSkin"
				@add-skin="openAddSkinFileBrowser"
				@add-skin-dragenter="onAddSkinDragOver"
				@add-skin-dragover="onAddSkinDragOver"
				@add-skin-dragleave="onAddSkinDragLeave"
				@add-skin-drop="onAddSkinDrop"
			/>
		</div>
	</div>

	<div v-else class="box-border flex min-h-full items-center justify-center pt-[25%]">
		<div
			class="relative mx-auto flex w-full max-w-xl flex-col gap-5 rounded-lg bg-bg-raised p-7 shadow-lg"
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
					<button :disabled="accountsCard.loginDisabled" @click="login">
						<LogInIcon v-if="!accountsCard.loginDisabled" />
						<SpinnerIcon v-else class="animate-spin" />
						Sign In
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<style lang="scss" scoped>
.skin-layout {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(0, 2.5fr);
	gap: 2.5rem;

	@media (max-width: 700px) {
		grid-template-columns: 1fr;
	}
}
</style>
