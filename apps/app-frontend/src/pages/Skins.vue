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
	commonMessages,
	ConfirmModal,
	defineMessages,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	SkinPreviewRenderer,
	useVIntl,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { type DragDropEvent, getCurrentWebview } from '@tauri-apps/api/webview'
import { computedAsync } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, inject, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'

import type AccountsCard from '@/components/ui/AccountsCard.vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import VirtualSkinSectionList from '@/components/ui/skin/VirtualSkinSectionList.vue'
import { trackEvent } from '@/helpers/analytics'
import { check_reachable, get_default_user, login as login_flow, users } from '@/helpers/auth'
import type { RenderResult } from '@/helpers/rendering/batch-skin-renderer.ts'
import { generateSkinPreviews, skinBlobUrlMap } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Cape, Skin, SkinTextureUrl } from '@/helpers/skins.ts'
import {
	equip_skin,
	filterDefaultSkins,
	filterSavedSkins,
	flush_pending_skin_change,
	flush_pending_skin_change_for_profile,
	get_available_capes,
	get_available_skins,
	get_dragged_skin_data,
	get_normalized_skin_texture,
	normalize_skin_texture,
	remove_custom_skin,
	save_custom_skin,
	set_custom_skin_order,
} from '@/helpers/skins.ts'
import { hasPride26Badge } from '@/helpers/user-campaigns.ts'
import { handleSevereError } from '@/store/error'
import { useTheming } from '@/store/state'

type UnlistenFn = () => void
type VirtualSkinSectionListExpose = {
	getAddSkinButtonElement: () => HTMLElement | null | undefined
}

const PENDING_SKIN_REFRESH_DELAY_MS = 11_000
const DEFAULT_SKIN_SECTION_SORT_ORDER = ['Default skins', 'Modrinth Pride']
const messages = defineMessages({
	skinSelectorTitle: {
		id: 'app.skins.title',
		defaultMessage: 'Skin selector',
	},
	modrinthPrideSection: {
		id: 'app.skins.section.modrinth-pride',
		defaultMessage: 'Modrinth Pride',
	},
	modrinthPrideTooltip: {
		id: 'app.skins.section.modrinth-pride.tooltip',
		defaultMessage:
			'You received these skins for donating to a Modrinth Pride fundraiser during Pride Month.',
	},
	modrinthSection: {
		id: 'app.skins.section.modrinth',
		defaultMessage: 'Modrinth',
	},
	defaultSkinsSection: {
		id: 'app.skins.section.default-skins',
		defaultMessage: 'Default skins',
	},
	mineconEarth2017Section: {
		id: 'app.skins.section.minecon-earth-2017',
		defaultMessage: 'MINECON Earth 2017',
	},
	buildersAndBiomesSection: {
		id: 'app.skins.section.builders-and-biomes',
		defaultMessage: 'Builders & Biomes',
	},
	stridingHeroSection: {
		id: 'app.skins.section.striding-hero',
		defaultMessage: 'Striding Hero',
	},
	theGardenAwakensSection: {
		id: 'app.skins.section.the-garden-awakens',
		defaultMessage: 'The Garden Awakens',
	},
	chaseTheSkiesSection: {
		id: 'app.skins.section.chase-the-skies',
		defaultMessage: 'Chase the Skies',
	},
	theCopperAgeSection: {
		id: 'app.skins.section.the-copper-age',
		defaultMessage: 'The Copper Age',
	},
	mountsOfMayhemSection: {
		id: 'app.skins.section.mounts-of-mayhem',
		defaultMessage: 'Mounts of Mayhem',
	},
	tinyTakeoverSection: {
		id: 'app.skins.section.tiny-takeover',
		defaultMessage: 'Tiny Takeover',
	},
	chaosCubedSection: {
		id: 'app.skins.section.chaos-cubed',
		defaultMessage: 'Chaos Cubed',
	},
	rateLimitTitle: {
		id: 'app.skins.rate-limit.title',
		defaultMessage: 'Slow down!',
	},
	rateLimitText: {
		id: 'app.skins.rate-limit.text',
		defaultMessage:
			"You're changing your skin too frequently. Mojang's servers have temporarily blocked further requests. Please wait a moment before trying again.",
	},
	droppedFileErrorTitle: {
		id: 'app.skins.dropped-file-error.title',
		defaultMessage: 'Error processing file',
	},
	droppedFileErrorText: {
		id: 'app.skins.dropped-file-error.text',
		defaultMessage: 'Failed to read the dropped file.',
	},
	reorderSkinErrorTitle: {
		id: 'app.skins.reorder-error.title',
		defaultMessage: 'Failed to reorder skins',
	},
	reorderSkinErrorText: {
		id: 'app.skins.reorder-error.text',
		defaultMessage: 'Your skin order could not be saved.',
	},
	deleteSkinTitle: {
		id: 'app.skins.delete-modal.title',
		defaultMessage: 'Are you sure you want to delete this skin?',
	},
	deleteSkinDescription: {
		id: 'app.skins.delete-modal.description',
		defaultMessage: 'This will permanently delete the selected skin. This action cannot be undone.',
	},
	previewingBadge: {
		id: 'app.skins.previewing-badge',
		defaultMessage: 'Previewing',
	},
	applyButton: {
		id: 'app.skins.apply-button',
		defaultMessage: 'Apply',
	},
	editSkinButton: {
		id: 'app.skins.preview.edit-button',
		defaultMessage: 'Edit skin',
	},
	excitedRinthbotAlt: {
		id: 'app.skins.sign-in.rinthbot-alt',
		defaultMessage: 'Excited Modrinth Bot',
	},
	signInTitle: {
		id: 'app.skins.sign-in.title',
		defaultMessage: 'Please sign in',
	},
	signInDescription: {
		id: 'app.skins.sign-in.description',
		defaultMessage:
			'Please sign into your Minecraft account to use the skin management features of the Modrinth app.',
	},
	signInButton: {
		id: 'app.skins.sign-in.button',
		defaultMessage: 'Sign In',
	},
})

const editSkinModal = useTemplateRef('editSkinModal')
const addSkinFileInput = useTemplateRef<HTMLInputElement>('addSkinFileInput')
const skinSectionList = useTemplateRef<VirtualSkinSectionListExpose>('skinSectionList')

const { formatMessage } = useVIntl()
const notifications = injectNotificationManager()
const { addNotification, handleError } = notifications
const auth = injectAuth()
const client = injectModrinthClient()

const themeStore = useTheming()
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])
const offline = ref(!navigator.onLine)

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
const authServerQuery = useQuery({
	queryKey: ['authServerReachability'],
	queryFn: async () => {
		await check_reachable()
		return true
	},
	refetchInterval: 5 * 60 * 1000,
	retry: false,
	refetchOnWindowFocus: false,
})
const { data: modrinthUser } = useQuery({
	queryKey: computed(() => ['authenticated-user', 'campaigns', auth.user.value?.id]),
	queryFn: () => client.labrinth.users_v3.getAuthenticated(),
	enabled: () => !!auth.session_token.value,
	retry: false,
})
const hasModrinthPrideCampaign = computed(
	() => !!auth.session_token.value && hasPride26Badge(modrinthUser.value?.campaigns?.pride_26),
)
const defaultSkins = computed(() =>
	filterDefaultSkins(skins.value).filter(
		(skin) => skin.section !== 'Modrinth Pride' || hasModrinthPrideCampaign.value,
	),
)
const defaultSkinSections = computed(() => {
	const sections = new Map<string, Skin[]>()

	for (const skin of defaultSkins.value) {
		const section = skin.section ?? 'Default skins'
		const sectionSkins = sections.get(section)

		if (sectionSkins) {
			sectionSkins.push(skin)
		} else {
			sections.set(section, [skin])
		}
	}

	return Array.from(sections, ([section, skins]) => ({
		section,
		title: getDefaultSkinSectionTitle(section),
		infoTooltip: getDefaultSkinSectionInfoTooltip(section),
		skins,
	})).sort(
		(a, b) => getDefaultSkinSectionSortIndex(a.section) - getDefaultSkinSectionSortIndex(b.section),
	)
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
	const skin = selectedSkin.value
	if (skin?.texture) {
		try {
			return await get_normalized_skin_texture(skin)
		} catch (error) {
			if (skin.texture.startsWith('data:image/')) {
				return skin.texture
			}

			handleError(error as Error)
			return ''
		}
	} else {
		return ''
	}
})
const capeTexture = computed(() => currentCape.value?.texture)
const skinVariant = computed(() => selectedSkin.value?.variant)
const skinNametag = computed(() => (themeStore.hideNametagSkinsPage ? undefined : username.value))
const isSkinManagementReadOnly = computed(
	() => offline.value || (authServerQuery.isError.value && !authServerQuery.isLoading.value),
)
const hasPendingSkinChange = computed(
	() => !skinsMatch(selectedSkin.value, originalSelectedSkin.value),
)

let userCheckInterval: number | null = null
let pendingSkinRefreshTimeout: number | null = null
let unlistenAddSkinDragDrop: UnlistenFn | null = null
let isUnmounted = false

const isDraggingSkinFile = ref(false)
const isAddSkinButtonDragActive = ref(false)

const deleteSkinModal = ref()
const skinToDelete = ref<Skin | null>(null)

function confirmDeleteSkin(skin: Skin) {
	if (isSkinManagementReadOnly.value) return

	skinToDelete.value = skin
	deleteSkinModal.value?.show()
}

async function deleteSkin() {
	if (isSkinManagementReadOnly.value) return

	const deletedSkin = skinToDelete.value
	if (!deletedSkin) return

	try {
		await remove_custom_skin(deletedSkin)
		removeLocalSkin(deletedSkin)
	} catch (error) {
		handleError(error as Error)
	} finally {
		skinToDelete.value = null
	}
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
		const loadedSkins = (await get_available_skins()) ?? []
		const loadedEquippedSkin = loadedSkins.find((s) => s.is_equipped)
		const locallyKnownEquippedSkin =
			originalSelectedSkin.value &&
			(loadedSkins.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
				(originalSelectedSkin.value.texture.startsWith('data:image/')
					? originalSelectedSkin.value
					: undefined))
		const shouldPreserveKnownEquippedSkin =
			isSkinManagementReadOnly.value &&
			locallyKnownEquippedSkin &&
			!skinsMatch(loadedEquippedSkin, locallyKnownEquippedSkin)

		skins.value =
			shouldPreserveKnownEquippedSkin && locallyKnownEquippedSkin
				? mergeEquippedSkin(loadedSkins, locallyKnownEquippedSkin)
				: loadedSkins
		generateSkinPreviews(skins.value, capes.value)
		selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null
		originalSelectedSkin.value = selectedSkin.value
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

function mergeEquippedSkin(list: Skin[], equippedSkin: Skin) {
	let foundEquippedSkin = false
	const mergedSkins = list.map((skin) => {
		const isEquipped = skinsMatch(skin, equippedSkin)
		foundEquippedSkin ||= isEquipped

		return {
			...skin,
			is_equipped: isEquipped,
		}
	})

	if (!foundEquippedSkin) {
		mergedSkins.unshift({
			...equippedSkin,
			is_equipped: true,
		})
	}

	return mergedSkins
}

function skinsMatch(a?: Skin | null, b?: Skin | null) {
	return (
		a?.source === b?.source &&
		a?.texture_key === b?.texture_key &&
		a?.variant === b?.variant &&
		(a?.cape_id ?? null) === (b?.cape_id ?? null)
	)
}

function skinsMatchIgnoringSource(a?: Skin | null, b?: Skin | null) {
	return (
		a?.texture_key === b?.texture_key &&
		a?.variant === b?.variant &&
		(a?.cape_id ?? null) === (b?.cape_id ?? null)
	)
}

function isSkinSelected(skin: Skin) {
	return skinsMatch(selectedSkin.value, skin)
}

function isSkinActive(skin: Skin) {
	return hasPendingSkinChange.value && skinsMatch(originalSelectedSkin.value, skin)
}

function getErrorMessage(error: unknown) {
	return error instanceof Error ? error.message : String(error)
}

function isMinecraftSkinRateLimitError(error: unknown) {
	const message = getErrorMessage(error)
	return message.includes('429 Too Many Requests') || message.includes('client error (429')
}

function getDefaultSkinSectionTitle(section?: string) {
	switch (section) {
		case 'Modrinth Pride':
			return formatMessage(messages.modrinthPrideSection)
		case 'Modrinth':
			return formatMessage(messages.modrinthSection)
		case 'MINECON Earth 2017':
			return formatMessage(messages.mineconEarth2017Section)
		case 'Builders & Biomes':
			return formatMessage(messages.buildersAndBiomesSection)
		case 'Striding Hero':
			return formatMessage(messages.stridingHeroSection)
		case 'The Garden Awakens':
			return formatMessage(messages.theGardenAwakensSection)
		case 'Chase the Skies':
			return formatMessage(messages.chaseTheSkiesSection)
		case 'The Copper Age':
			return formatMessage(messages.theCopperAgeSection)
		case 'Mounts of Mayhem':
			return formatMessage(messages.mountsOfMayhemSection)
		case 'Tiny Takeover':
			return formatMessage(messages.tinyTakeoverSection)
		case 'Chaos Cubed':
			return formatMessage(messages.chaosCubedSection)
		case 'Default skins':
			return formatMessage(messages.defaultSkinsSection)
		default:
			return section ?? formatMessage(messages.defaultSkinsSection)
	}
}

function getDefaultSkinSectionInfoTooltip(section: string) {
	switch (section) {
		case 'Modrinth Pride':
			return formatMessage(messages.modrinthPrideTooltip)
		default:
			return undefined
	}
}

function getDefaultSkinSectionSortIndex(section: string) {
	const index = DEFAULT_SKIN_SECTION_SORT_ORDER.indexOf(section)
	return index === -1 ? DEFAULT_SKIN_SECTION_SORT_ORDER.length : index
}

function changeSkin(newSkin: Skin) {
	if (isSkinManagementReadOnly.value) return

	selectedSkin.value = newSkin
}

function resetSelectedSkin() {
	selectedSkin.value =
		skins.value.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
		originalSelectedSkin.value
}

function removeLocalSkin(deletedSkin: Skin) {
	const nextSkins = skins.value.filter((skin) => !skinsMatch(skin, deletedSkin))
	skins.value = nextSkins

	if (selectedSkin.value && skinsMatch(selectedSkin.value, deletedSkin)) {
		selectedSkin.value =
			nextSkins.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
			nextSkins.find((skin) => skin.is_equipped) ??
			null
	}

	if (originalSelectedSkin.value && skinsMatch(originalSelectedSkin.value, deletedSkin)) {
		originalSelectedSkin.value = nextSkins.find((skin) => skin.is_equipped) ?? null
	}

	generateSkinPreviews(skins.value, capes.value)
}

function setLocallyEquippedSkin(skinToApply: Skin) {
	skins.value = skins.value.map((skin) => ({
		...skin,
		is_equipped: skinsMatch(skin, skinToApply),
	}))
	originalSelectedSkin.value =
		skins.value.find((skin) => skinsMatch(skin, skinToApply)) ?? skinToApply
	selectedSkin.value = originalSelectedSkin.value
	void accountsCard.value?.setEquippedSkin(originalSelectedSkin.value)
}

function insertLocalSkin(savedSkin: Skin) {
	const firstNonCustomSkinIndex = skins.value.findIndex((skin) => skin.source !== 'custom')

	if (firstNonCustomSkinIndex === -1) {
		skins.value = [...skins.value, savedSkin]
		return
	}

	const nextSkins = [...skins.value]
	nextSkins.splice(firstNonCustomSkinIndex, 0, savedSkin)
	skins.value = nextSkins
}

function updateLocalSkin(savedSkin: Skin, applied: boolean, previousSkin?: Skin) {
	let foundSkin = false
	const replacesSelectedSkin =
		selectedSkin.value?.texture_key === savedSkin.texture_key ||
		(previousSkin ? skinsMatch(selectedSkin.value, previousSkin) : false)
	const replacesOriginalSkin =
		originalSelectedSkin.value?.texture_key === savedSkin.texture_key ||
		(previousSkin ? skinsMatch(originalSelectedSkin.value, previousSkin) : false)

	skins.value = skins.value.map((skin) => {
		const isUpdatedSkin = skin.texture_key === savedSkin.texture_key
		const isPreviousSkin = previousSkin && skinsMatch(skin, previousSkin)

		if (isUpdatedSkin || isPreviousSkin) {
			foundSkin = true
			return {
				...savedSkin,
				is_equipped: applied || savedSkin.is_equipped,
			}
		}

		return {
			...skin,
			is_equipped: applied ? false : skin.is_equipped,
		}
	})

	if (!foundSkin) {
		insertLocalSkin({
			...savedSkin,
			is_equipped: applied || savedSkin.is_equipped,
		})
	}

	if (applied) {
		const locallyEquippedSkin =
			skins.value.find((skin) => skin.texture_key === savedSkin.texture_key) ?? savedSkin

		originalSelectedSkin.value = locallyEquippedSkin
		selectedSkin.value = locallyEquippedSkin
		void accountsCard.value?.setEquippedSkin(locallyEquippedSkin)
	} else {
		const locallySavedSkin =
			skins.value.find((skin) => skin.texture_key === savedSkin.texture_key) ?? savedSkin

		if (replacesSelectedSkin) {
			selectedSkin.value = locallySavedSkin
		}

		if (replacesOriginalSkin) {
			originalSelectedSkin.value = locallySavedSkin
		}
	}

	generateSkinPreviews(skins.value, capes.value)
}

async function reorderSavedSkins(orderedSkins: Skin[]) {
	const previousSkins = skins.value
	const previousSelectedSkin = selectedSkin.value
	const previousOriginalSelectedSkin = originalSelectedSkin.value
	const orderedTextureKeys = orderedSkins.map((skin) => skin.texture_key)
	const orderedTextureKeySet = new Set(orderedTextureKeys)
	const remainingSavedSkins = previousSkins.filter(
		(skin) => skin.source !== 'default' && !orderedTextureKeySet.has(skin.texture_key),
	)
	const defaultSkins = previousSkins.filter((skin) => skin.source === 'default')
	const nextSavedSkins = [...orderedSkins, ...remainingSavedSkins]

	skins.value = [...nextSavedSkins, ...defaultSkins]
	generateSkinPreviews(skins.value, capes.value)

	try {
		const persistedSavedSkins = await preserveExternalSkins(nextSavedSkins)

		if (persistedSavedSkins.some((skin, index) => skin !== nextSavedSkins[index])) {
			skins.value = [...persistedSavedSkins, ...defaultSkins]
			generateSkinPreviews(skins.value, capes.value)
		}

		await set_custom_skin_order(
			persistedSavedSkins
				.filter((skin) => skin.source === 'custom')
				.map((skin) => skin.texture_key),
		)
	} catch (error) {
		skins.value = previousSkins
		selectedSkin.value = previousSelectedSkin
		originalSelectedSkin.value = previousOriginalSelectedSkin
		generateSkinPreviews(skins.value, capes.value)
		addNotification({
			type: 'error',
			title: formatMessage(messages.reorderSkinErrorTitle),
			text: error instanceof Error ? error.message : formatMessage(messages.reorderSkinErrorText),
		})
		await loadSkins()
	}
}

async function preserveExternalSkins(skinsToPersist: Skin[]) {
	const preservedSkins: Skin[] = []

	for (const skin of skinsToPersist) {
		if (skin.source !== 'custom_external') {
			preservedSkins.push(skin)
			continue
		}

		const textureBlob = await normalize_skin_texture(skin.texture)
		const capeId = skin.cape_id ? capes.value.find((cape) => cape.id === skin.cape_id) : undefined
		const savedSkin = await save_custom_skin(skin, textureBlob, skin.variant, capeId, false)
		const preservedSkin: Skin = {
			...savedSkin,
			source: 'custom',
			is_equipped: skin.is_equipped,
		}

		if (skinsMatchIgnoringSource(selectedSkin.value, skin)) {
			selectedSkin.value = preservedSkin
		}

		if (skinsMatchIgnoringSource(originalSelectedSkin.value, skin)) {
			originalSelectedSkin.value = preservedSkin
			void accountsCard.value?.setEquippedSkin(preservedSkin)
		}

		preservedSkins.push(preservedSkin)
	}

	return preservedSkins
}

function schedulePendingSkinRefresh() {
	if (pendingSkinRefreshTimeout !== null) {
		window.clearTimeout(pendingSkinRefreshTimeout)
	}

	const pendingProfileId = currentUserId.value

	pendingSkinRefreshTimeout = window.setTimeout(async () => {
		pendingSkinRefreshTimeout = null

		if (isUnmounted) {
			return
		}

		try {
			if (pendingProfileId) {
				await flush_pending_skin_change_for_profile(pendingProfileId)
			} else {
				await flush_pending_skin_change()
			}
		} catch (error) {
			handleError(error as Error)
			schedulePendingSkinRefresh()
			return
		}

		if (accountsCard.value) {
			await accountsCard.value.refreshValues()
		}

		await loadCapes()
		await loadSkins()
	}, PENDING_SKIN_REFRESH_DELAY_MS)
}

async function applySelectedSkin() {
	const skinToApply = selectedSkin.value
	if (
		!skinToApply ||
		!hasPendingSkinChange.value ||
		isApplyingSkin.value ||
		isSkinManagementReadOnly.value
	)
		return

	isApplyingSkin.value = true
	try {
		await equip_skin(skinToApply)
		setLocallyEquippedSkin(skinToApply)
		schedulePendingSkinRefresh()
	} catch (error) {
		if (isMinecraftSkinRateLimitError(error)) {
			notifications.addNotification({
				type: 'error',
				title: formatMessage(messages.rateLimitTitle),
				text: formatMessage(messages.rateLimitText),
			})
		} else {
			handleError(error as Error)
		}
	} finally {
		isApplyingSkin.value = false
	}
}

async function onSkinSaved(options: { applied: boolean; skin?: Skin; previousSkin?: Skin }) {
	if (options.skin) {
		updateLocalSkin(options.skin, options.applied, options.previousSkin)
	}

	if (!options.skin) {
		await loadCapes()
		await loadSkins()
	}

	if (options.applied) {
		schedulePendingSkinRefresh()
	}
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
	if (isSkinManagementReadOnly.value) return

	addSkinFileInput.value?.click()
}

async function onAddSkinFileInputChange(e: Event) {
	if (isSkinManagementReadOnly.value) return

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
	if (isSkinManagementReadOnly.value) return

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
			title: formatMessage(messages.droppedFileErrorTitle),
			text: error instanceof Error ? error.message : formatMessage(messages.droppedFileErrorText),
			type: 'error',
		})
	}
}

function onAddSkinDragOver(event: DragEvent) {
	if (isSkinManagementReadOnly.value) return

	if (!isSkinFileDrag(event)) {
		return
	}

	isAddSkinButtonDragActive.value = true
}

function onAddSkinDragLeave() {
	if (isSkinManagementReadOnly.value) return

	isAddSkinButtonDragActive.value = false
}

async function onAddSkinDrop(event: DragEvent) {
	if (isSkinManagementReadOnly.value) return

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
	if (isSkinManagementReadOnly.value) return

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

watch(isSkinManagementReadOnly, (readOnly) => {
	if (readOnly) {
		isDraggingSkinFile.value = false
		isAddSkinButtonDragActive.value = false
	}
})

onMounted(() => {
	window.addEventListener('offline', onOffline)
	window.addEventListener('online', onOnline)
	userCheckInterval = window.setInterval(checkUserChanges, 250)
	void setupAddSkinDragDropListener()
})

onUnmounted(() => {
	isUnmounted = true
	window.removeEventListener('offline', onOffline)
	window.removeEventListener('online', onOnline)

	if (userCheckInterval !== null) {
		window.clearInterval(userCheckInterval)
	}

	if (pendingSkinRefreshTimeout !== null) {
		window.clearTimeout(pendingSkinRefreshTimeout)
		pendingSkinRefreshTimeout = null
	}

	if (unlistenAddSkinDragDrop) {
		unlistenAddSkinDragDrop()
		unlistenAddSkinDragDrop = null
	}
})

function onOffline() {
	offline.value = true
}

function onOnline() {
	offline.value = false
	void authServerQuery.refetch()
}

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
		:title="formatMessage(messages.deleteSkinTitle)"
		:description="formatMessage(messages.deleteSkinDescription)"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		@proceed="deleteSkin"
	/>

	<div v-if="currentUser" class="skin-layout box-border min-h-full p-4">
		<div class="sticky top-6 self-start p-2 pt-0">
			<h1 class="m-0 text-2xl font-bold flex items-center gap-2">
				{{ formatMessage(messages.skinSelectorTitle) }}
			</h1>
			<div
				class="ml-5 mt-4 flex h-[calc(80vh-1rem)] items-center justify-center max-[700px]:h-[calc(50vh-1rem)]"
			>
				<SkinPreviewRenderer
					:cape-src="capeTexture"
					:texture-src="skinTexture || ''"
					:variant="skinVariant"
					:nametag="skinNametag"
					:initial-rotation="Math.PI / 8"
				>
					<template v-if="hasPendingSkinChange" #nametag-badge>
						<div
							class="flex items-center justify-center gap-1.5 rounded-full border border-solid border-brand-blue bg-bg-blue px-3 py-1 text-base font-semibold leading-6 text-brand-blue"
						>
							<EyeIcon class="size-5 shrink-0" />
							{{ formatMessage(messages.previewingBadge) }}
						</div>
					</template>
					<template #subtitle>
						<div
							v-if="hasPendingSkinChange"
							class="flex max-w-[calc(100vw-2rem)] flex-wrap items-center justify-center gap-2 px-2"
						>
							<button
								class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-2 rounded-[14px] border-0 bg-surface-4 px-4 py-2.5 text-base font-semibold leading-5 text-contrast shadow-md transition-[filter,transform] duration-200 enabled:hover:brightness-[--hover-brightness] enabled:focus-visible:brightness-[--hover-brightness] enabled:active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-5 [&>svg]:shrink-0"
								:disabled="isApplyingSkin || isSkinManagementReadOnly"
								@click="resetSelectedSkin"
							>
								<RotateCounterClockwiseIcon />
								{{ formatMessage(commonMessages.resetButton) }}
							</button>
							<button
								class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-2 rounded-[14px] border-0 bg-brand px-4 py-2.5 text-base font-semibold leading-5 text-[rgba(0,0,0,0.9)] shadow-md transition-[filter,transform] duration-200 enabled:hover:brightness-[--hover-brightness] enabled:focus-visible:brightness-[--hover-brightness] enabled:active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-5 [&>svg]:shrink-0"
								:disabled="isApplyingSkin || isSkinManagementReadOnly"
								@click="applySelectedSkin"
							>
								<SpinnerIcon v-if="isApplyingSkin" class="animate-spin" />
								<CheckIcon v-else />
								{{ formatMessage(messages.applyButton) }}
							</button>
						</div>
						<button
							v-else
							class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-2 rounded-[14px] border-0 bg-surface-4 px-4 py-2.5 text-base font-semibold leading-5 shadow-md transition-[filter,transform] duration-200 enabled:hover:brightness-[--hover-brightness] enabled:focus-visible:brightness-[--hover-brightness] enabled:active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-5 [&>svg]:shrink-0"
							:disabled="!selectedSkin || isSkinManagementReadOnly"
							@click="(e: MouseEvent) => selectedSkin && editSkinModal?.show(e, selectedSkin)"
						>
							<EditIcon />
							{{ formatMessage(messages.editSkinButton) }}
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
				:is-skin-active="isSkinActive"
				:is-add-skin-button-drag-active="isAddSkinButtonDragActive"
				:read-only="isSkinManagementReadOnly"
				@select="changeSkin"
				@edit="(skin, event) => editSkinModal?.show(event, skin)"
				@delete="confirmDeleteSkin"
				@reorder-saved-skins="reorderSavedSkins"
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
				:alt="formatMessage(messages.excitedRinthbotAlt)"
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
				<h1 class="text-3xl font-extrabold m-0">{{ formatMessage(messages.signInTitle) }}</h1>
				<p class="text-lg m-0">
					{{ formatMessage(messages.signInDescription) }}
				</p>
				<ButtonStyled v-show="accountsCard" color="brand" :disabled="accountsCard.loginDisabled">
					<button :disabled="accountsCard.loginDisabled" @click="login">
						<LogInIcon v-if="!accountsCard.loginDisabled" />
						<SpinnerIcon v-else class="animate-spin" />
						{{ formatMessage(messages.signInButton) }}
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
