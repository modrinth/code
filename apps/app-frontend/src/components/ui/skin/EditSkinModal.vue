<template>
	<NewModal ref="modal" :on-hide="handleModalHide">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">
				{{ mode === 'edit' ? 'Editing skin' : 'Adding a skin' }}
			</span>
		</template>

		<div class="flex flex-col md:flex-row gap-6">
			<div class="h-[25rem] w-[16rem] min-w-[16rem] flex-shrink-0 md:self-center">
				<SkinPreviewRenderer
					:variant="variant"
					:texture-src="previewSkin || ''"
					:cape-src="selectedCapeTexture"
					framing="modal"
					:initial-rotation="Math.PI / 8"
					class="h-full w-full"
				/>
			</div>

			<div class="flex flex-col gap-4 w-full min-h-[20rem]">
				<section v-if="mode === 'edit' && canEditTextureAndModel">
					<h2 class="text-base font-semibold mb-2">Texture</h2>
					<ButtonStyled>
						<button @click="openTextureFileBrowser" class="!shadow-none">
							<UploadIcon /> Replace texture
						</button>
					</ButtonStyled>
					<input
						ref="textureFileInput"
						type="file"
						accept="image/png"
						class="hidden"
						@change="onTextureFileInputChange"
					/>
				</section>

				<section v-if="canEditTextureAndModel">
					<h2 class="text-base font-semibold mb-2">Arm style</h2>
					<RadioButtons v-model="variant" :items="['CLASSIC', 'SLIM']">
						<template #default="{ item }">
							{{ item === 'CLASSIC' ? 'Wide' : 'Slim' }}
						</template>
					</RadioButtons>
				</section>

				<section>
					<h2 class="text-base font-semibold mb-2">Cape</h2>
					<div class="relative w-fit max-w-full">
						<Transition
							enter-active-class="transition-all duration-200 ease-out"
							enter-from-class="opacity-0 max-h-0"
							enter-to-class="opacity-100 max-h-6"
							leave-active-class="transition-all duration-200 ease-in"
							leave-from-class="opacity-100 max-h-6"
							leave-to-class="opacity-0 max-h-0"
						>
							<div
								v-if="showCapeTopFade"
								class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-6 bg-gradient-to-b from-bg-raised to-transparent"
							/>
						</Transition>

						<div
							ref="capeListRef"
							class="grid grid-cols-[repeat(4,max-content)] auto-rows-max gap-2 overflow-y-auto pr-1"
							:style="{ maxHeight: capeListMaxHeight }"
							@scroll="checkCapeScrollState"
						>
							<CapeLikeTextButton
								tooltip="No cape"
								:highlighted="!selectedCape"
								@click="selectCape(undefined)"
							>
								<template #icon><XIcon /></template>
								<span>None</span>
							</CapeLikeTextButton>

							<CapeButton
								v-for="cape in sortedCapes"
								:id="cape.id"
								:key="cape.id"
								:texture="cape.texture"
								:name="cape.name || 'Cape'"
								:selected="selectedCape?.id === cape.id"
								@select="selectCape(cape)"
							/><CapeButton
								v-for="cape in sortedCapes"
								:id="cape.id"
								:key="cape.id"
								:texture="cape.texture"
								:name="cape.name || 'Cape'"
								:selected="selectedCape?.id === cape.id"
								@select="selectCape(cape)"
							/>
						</div>

						<Transition
							enter-active-class="transition-all duration-200 ease-out"
							enter-from-class="opacity-0 max-h-0"
							enter-to-class="opacity-100 max-h-6"
							leave-active-class="transition-all duration-200 ease-in"
							leave-from-class="opacity-100 max-h-6"
							leave-to-class="opacity-0 max-h-0"
						>
							<div
								v-if="showCapeBottomFade"
								class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-6 bg-gradient-to-t from-bg-raised to-transparent"
							/>
						</Transition>
					</div>
				</section>
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button :disabled="isSaving" @click="hide"><XIcon />Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button v-tooltip="saveTooltip" :disabled="disableSave || isSaving" @click="save">
						<SpinnerIcon v-if="isSaving" class="animate-spin" />
						<CheckIcon v-else-if="mode === 'new'" />
						<SaveIcon v-else />
						{{ mode === 'new' ? 'Add skin' : 'Save skin' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, SaveIcon, SpinnerIcon, UploadIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	CapeButton,
	CapeLikeTextButton,
	injectNotificationManager,
	NewModal,
	RadioButtons,
	SkinPreviewRenderer,
	useScrollIndicator,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import {
	add_and_equip_custom_skin,
	type Cape,
	determineModelType,
	get_normalized_skin_texture,
	normalize_skin_texture,
	remove_custom_skin,
	type Skin,
	type SkinModel,
	type SkinTextureUrl,
	unequip_skin,
} from '@/helpers/skins.ts'

const CAPE_LIST_MAX_HEIGHT = 334

const { handleError } = injectNotificationManager()

const modal = useTemplateRef('modal')
const textureFileInput = useTemplateRef<HTMLInputElement>('textureFileInput')
const capeListRef = ref<HTMLElement | null>(null)
const capeListMaxHeight = ref(`${CAPE_LIST_MAX_HEIGHT}px`)
const mode = ref<'new' | 'edit'>('new')
const currentSkin = ref<Skin | null>(null)
const isSaving = ref(false)

const uploadedTextureUrl = ref<SkinTextureUrl | null>(null)
const previewSkin = ref<string>('')

const variant = ref<SkinModel>('CLASSIC')
const selectedCape = ref<Cape | undefined>(undefined)
const props = defineProps<{ capes?: Cape[] }>()

const selectedCapeTexture = computed(() => selectedCape.value?.texture)
const canEditTextureAndModel = computed(() => currentSkin.value?.source !== 'default')
const {
	showTopFade: showCapeTopFade,
	showBottomFade: showCapeBottomFade,
	checkScrollState: checkCapeScrollState,
	forceCheck: forceCapeScrollCheck,
} = useScrollIndicator(capeListRef)

let capeListLayoutFrame: number | null = null
function updateCapeListLayout() {
	const capeList = capeListRef.value
	const modalContent = capeList?.closest('[data-modal-content]') as HTMLElement | null

	if (!capeList || !modalContent) {
		capeListMaxHeight.value = `${CAPE_LIST_MAX_HEIGHT}px`
		forceCapeScrollCheck()
		return
	}

	const availableHeight =
		modalContent.getBoundingClientRect().bottom - capeList.getBoundingClientRect().top

	capeListMaxHeight.value = `${Math.min(
		CAPE_LIST_MAX_HEIGHT,
		Math.max(0, Math.floor(availableHeight)),
	)}px`

	nextTick(() => forceCapeScrollCheck())
}

function refreshCapeListLayout() {
	if (capeListLayoutFrame !== null) {
		cancelAnimationFrame(capeListLayoutFrame)
	}

	capeListLayoutFrame = requestAnimationFrame(() => {
		capeListLayoutFrame = null
		updateCapeListLayout()
	})
}

const sortedCapes = computed(() => {
	return [...(props.capes || [])].sort((a, b) => {
		const nameA = (a.name || '').toLowerCase()
		const nameB = (b.name || '').toLowerCase()
		return nameA.localeCompare(nameB)
	})
})

async function loadPreviewSkin() {
	if (uploadedTextureUrl.value) {
		previewSkin.value = uploadedTextureUrl.value.normalized
	} else if (currentSkin.value) {
		try {
			previewSkin.value = await get_normalized_skin_texture(currentSkin.value)
		} catch (error) {
			console.error('Failed to load skin texture:', error)
			previewSkin.value = '/src/assets/skins/steve.png'
		}
	} else {
		previewSkin.value = '/src/assets/skins/steve.png'
	}
}

const hasEdits = computed(() => {
	if (mode.value !== 'edit') return true
	if (uploadedTextureUrl.value) return true
	if (!currentSkin.value) return false
	if (variant.value !== currentSkin.value.variant) return true
	if ((selectedCape.value?.id || null) !== (currentSkin.value.cape_id || null)) return true
	return false
})

const disableSave = computed(
	() =>
		(mode.value === 'new' && !uploadedTextureUrl.value) ||
		(mode.value === 'edit' && !hasEdits.value),
)

const saveTooltip = computed(() => {
	if (isSaving.value) return 'Saving...'
	if (mode.value === 'new' && !uploadedTextureUrl.value) return 'Upload a skin first!'
	if (mode.value === 'edit' && !hasEdits.value) return 'Make an edit to the skin first!'
	return undefined
})

function resetState() {
	mode.value = 'new'
	currentSkin.value = null
	uploadedTextureUrl.value = null
	previewSkin.value = ''
	variant.value = 'CLASSIC'
	selectedCape.value = undefined
	isSaving.value = false
}

function handleModalHide() {
	setTimeout(() => resetState(), 250)
}

async function show(e: MouseEvent, skin?: Skin) {
	mode.value = skin ? 'edit' : 'new'
	currentSkin.value = skin ?? null
	if (skin) {
		variant.value = skin.variant
		selectedCape.value = props.capes?.find((c) => c.id === skin.cape_id)
	} else {
		variant.value = 'CLASSIC'
		selectedCape.value = undefined
	}

	await loadPreviewSkin()

	modal.value?.show(e)
	nextTick(() => refreshCapeListLayout())
}

async function showNew(e: MouseEvent, skinTextureUrl: SkinTextureUrl) {
	mode.value = 'new'
	currentSkin.value = null
	uploadedTextureUrl.value = skinTextureUrl
	variant.value = await determineModelType(skinTextureUrl.original)
	selectedCape.value = undefined

	await loadPreviewSkin()

	modal.value?.show(e)
	nextTick(() => refreshCapeListLayout())
}

async function setUploadedTexture(skinTextureUrl: SkinTextureUrl) {
	uploadedTextureUrl.value = skinTextureUrl
	await loadPreviewSkin()
	nextTick(() => refreshCapeListLayout())
}

function hide() {
	modal.value?.hide()
}

function selectCape(cape: Cape | undefined) {
	selectedCape.value = cape
}

function openTextureFileBrowser() {
	textureFileInput.value?.click()
}

async function onTextureFileInputChange(e: Event) {
	const files = (e.target as HTMLInputElement).files
	const file = files?.[0]

	if (!file) {
		return
	}

	try {
		const originalSkinTexUrl = `data:image/png;base64,${arrayBufferToBase64(
			await file.arrayBuffer(),
		)}`
		const skinTextureNormalized = await normalize_skin_texture(originalSkinTexUrl)
		await setUploadedTexture({
			original: originalSkinTexUrl,
			normalized: `data:image/png;base64,${arrayBufferToBase64(skinTextureNormalized)}`,
		})
	} catch (error) {
		handleError(error)
	} finally {
		if (textureFileInput.value) {
			textureFileInput.value.value = ''
		}
	}
}

async function save() {
	isSaving.value = true

	try {
		let textureUrl: string

		if (uploadedTextureUrl.value) {
			textureUrl = uploadedTextureUrl.value.original
		} else {
			textureUrl = currentSkin.value!.texture
		}

		await unequip_skin()

		const bytes: Uint8Array = new Uint8Array(await (await fetch(textureUrl)).arrayBuffer())

		if (mode.value === 'new') {
			await add_and_equip_custom_skin(bytes, variant.value, selectedCape.value)
			emit('saved')
		} else {
			await add_and_equip_custom_skin(bytes, variant.value, selectedCape.value)
			if (uploadedTextureUrl.value && textureUrl !== currentSkin.value?.texture) {
				await remove_custom_skin(currentSkin.value!)
			}
			emit('saved')
		}

		hide()
	} catch (err) {
		handleError(err)
	} finally {
		isSaving.value = false
	}
}

watch([uploadedTextureUrl, currentSkin], async () => {
	await loadPreviewSkin()
	refreshCapeListLayout()
})

watch(
	() => props.capes,
	() => {
		nextTick(() => refreshCapeListLayout())
	},
	{ immediate: true },
)

watch(
	capeListRef,
	(capeList, _, onCleanup) => {
		if (!capeList) return

		const modalContent = capeList.closest('[data-modal-content]')
		const resizeObserver = new ResizeObserver(() => refreshCapeListLayout())

		if (modalContent instanceof HTMLElement) {
			resizeObserver.observe(modalContent)
		}

		window.addEventListener('resize', refreshCapeListLayout, { passive: true })
		refreshCapeListLayout()

		onCleanup(() => {
			resizeObserver.disconnect()
			window.removeEventListener('resize', refreshCapeListLayout)

			if (capeListLayoutFrame !== null) {
				cancelAnimationFrame(capeListLayoutFrame)
				capeListLayoutFrame = null
			}
		})
	},
	{ flush: 'post' },
)

const emit = defineEmits<{
	(event: 'saved'): void
	(event: 'deleted', skin: Skin): void
}>()

defineExpose({
	show,
	showNew,
	hide,
})
</script>
