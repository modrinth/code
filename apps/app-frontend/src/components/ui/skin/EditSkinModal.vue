<template>
	<NewModal ref="modal" :on-hide="handleModalHide">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">
				{{ formatMessage(mode === 'edit' ? messages.editSkinTitle : messages.addSkinTitle) }}
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
					<h2 class="text-base font-semibold mb-2">{{ formatMessage(messages.textureSection) }}</h2>
					<ButtonStyled>
						<button class="!shadow-none" @click="openTextureFileBrowser">
							<UploadIcon /> {{ formatMessage(messages.replaceTextureButton) }}
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
					<h2 class="text-base font-semibold mb-2">
						{{ formatMessage(messages.armStyleSection) }}
					</h2>
					<RadioButtons v-model="variant" :items="['CLASSIC', 'SLIM']">
						<template #default="{ item }">
							{{
								formatMessage(item === 'CLASSIC' ? messages.wideArmStyle : messages.slimArmStyle)
							}}
						</template>
					</RadioButtons>
				</section>

				<section>
					<h2 class="text-base font-semibold mb-2">{{ formatMessage(messages.capeSection) }}</h2>
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
								:tooltip="formatMessage(messages.noCapeTooltip)"
								:highlighted="!selectedCape"
								@click="selectCape(undefined)"
							>
								<template #icon><XIcon /></template>
								<span>{{ formatMessage(messages.noneCapeOption) }}</span>
							</CapeLikeTextButton>

							<CapeButton
								v-for="cape in sortedCapes"
								:id="cape.id"
								:key="cape.id"
								:texture="cape.texture"
								:name="cape.name || formatMessage(messages.capeFallbackName)"
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
					<button :disabled="isSaving" @click="hide">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button v-tooltip="saveTooltip" :disabled="disableSave || isSaving" @click="save">
						<SpinnerIcon v-if="isSaving" class="animate-spin" />
						<CheckIcon v-else-if="mode === 'new'" />
						<SaveIcon v-else />
						{{ formatMessage(mode === 'new' ? messages.addSkinButton : messages.saveSkinButton) }}
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
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	RadioButtons,
	SkinPreviewRenderer,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import {
	add_and_equip_custom_skin,
	type Cape,
	determineModelType,
	equip_skin,
	get_normalized_skin_texture,
	normalize_skin_texture,
	save_custom_skin,
	type Skin,
	type SkinModel,
	type SkinTextureUrl,
} from '@/helpers/skins.ts'

const CAPE_LIST_MAX_HEIGHT = 334
const messages = defineMessages({
	editSkinTitle: {
		id: 'app.skins.modal.edit-title',
		defaultMessage: 'Editing skin',
	},
	addSkinTitle: {
		id: 'app.skins.modal.add-title',
		defaultMessage: 'Adding a skin',
	},
	textureSection: {
		id: 'app.skins.modal.texture-section',
		defaultMessage: 'Texture',
	},
	replaceTextureButton: {
		id: 'app.skins.modal.replace-texture-button',
		defaultMessage: 'Replace texture',
	},
	armStyleSection: {
		id: 'app.skins.modal.arm-style-section',
		defaultMessage: 'Arm style',
	},
	wideArmStyle: {
		id: 'app.skins.modal.arm-style-wide',
		defaultMessage: 'Wide',
	},
	slimArmStyle: {
		id: 'app.skins.modal.arm-style-slim',
		defaultMessage: 'Slim',
	},
	capeSection: {
		id: 'app.skins.modal.cape-section',
		defaultMessage: 'Cape',
	},
	noCapeTooltip: {
		id: 'app.skins.modal.no-cape-tooltip',
		defaultMessage: 'No cape',
	},
	noneCapeOption: {
		id: 'app.skins.modal.none-cape-option',
		defaultMessage: 'None',
	},
	capeFallbackName: {
		id: 'app.skins.modal.cape-fallback-name',
		defaultMessage: 'Cape',
	},
	savingTooltip: {
		id: 'app.skins.modal.saving-tooltip',
		defaultMessage: 'Saving...',
	},
	uploadSkinFirstTooltip: {
		id: 'app.skins.modal.upload-skin-first-tooltip',
		defaultMessage: 'Upload a skin first!',
	},
	makeEditFirstTooltip: {
		id: 'app.skins.modal.make-edit-first-tooltip',
		defaultMessage: 'Make an edit to the skin first!',
	},
	addSkinButton: {
		id: 'app.skins.modal.add-skin-button',
		defaultMessage: 'Add skin',
	},
	saveSkinButton: {
		id: 'app.skins.modal.save-skin-button',
		defaultMessage: 'Save skin',
	},
})

const { formatMessage } = useVIntl()
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
	if (isSaving.value) return formatMessage(messages.savingTooltip)
	if (mode.value === 'new' && !uploadedTextureUrl.value) {
		return formatMessage(messages.uploadSkinFirstTooltip)
	}
	if (mode.value === 'edit' && !hasEdits.value) {
		return formatMessage(messages.makeEditFirstTooltip)
	}
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

		const bytes: Uint8Array = new Uint8Array(await (await fetch(textureUrl)).arrayBuffer())

		if (mode.value === 'new') {
			await add_and_equip_custom_skin(bytes, variant.value, selectedCape.value)
			emit('saved', { applied: true })
		} else {
			const updatedSkin = await save_custom_skin(
				currentSkin.value!,
				bytes,
				variant.value,
				selectedCape.value,
				!!uploadedTextureUrl.value && textureUrl !== currentSkin.value?.texture,
			)

			if (currentSkin.value?.is_equipped) {
				await equip_skin(updatedSkin)
			}

			emit('saved', {
				applied: !!currentSkin.value?.is_equipped,
				skin: updatedSkin,
			})
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
	(event: 'saved', options: { applied: boolean; skin?: Skin }): void
	(event: 'deleted', skin: Skin): void
}>()

defineExpose({
	show,
	showNew,
	hide,
})
</script>
