<template>
	<UploadSkinModal ref="uploadModal" />
	<ModalWrapper ref="modal" @on-hide="resetState">
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
					:scale="1.4"
					:fov="40"
					:initial-rotation="Math.PI / 8"
					class="h-full w-full"
				/>
			</div>

			<div class="flex flex-col gap-4 w-full min-h-[20rem]">
				<section v-if="mode === 'edit' && canEditTextureAndModel">
					<h2 class="text-base font-semibold mb-2">Texture</h2>
					<ButtonStyled>
						<button @click="openUploadSkinModal"><UploadIcon /> Replace texture</button>
					</ButtonStyled>
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
							class="grid max-h-[334px] grid-cols-[repeat(4,max-content)] auto-rows-max gap-2 overflow-y-auto pr-1"
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

		<div class="flex gap-2">
			<ButtonStyled color="brand">
				<button v-tooltip="saveTooltip" :disabled="disableSave || isSaving" @click="save">
					<SpinnerIcon v-if="isSaving" class="animate-spin" />
					<CheckIcon v-else-if="mode === 'new'" />
					<SaveIcon v-else />
					{{ mode === 'new' ? 'Add skin' : 'Save skin' }}
				</button>
			</ButtonStyled>
			<ButtonStyled type="outlined">
				<button :disabled="isSaving" @click="hide"><XIcon />Cancel</button>
			</ButtonStyled>
		</div>
	</ModalWrapper>
</template>

<script setup lang="ts">
import { CheckIcon, SaveIcon, SpinnerIcon, UploadIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	CapeButton,
	CapeLikeTextButton,
	injectNotificationManager,
	RadioButtons,
	SkinPreviewRenderer,
	useScrollIndicator,
} from '@modrinth/ui'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import UploadSkinModal from '@/components/ui/skin/UploadSkinModal.vue'
import {
	add_and_equip_custom_skin,
	type Cape,
	determineModelType,
	get_normalized_skin_texture,
	remove_custom_skin,
	type Skin,
	type SkinModel,
	type SkinTextureUrl,
	unequip_skin,
} from '@/helpers/skins.ts'

const { handleError } = injectNotificationManager()

const modal = useTemplateRef('modal')
const capeListRef = ref<HTMLElement | null>(null)
const mode = ref<'new' | 'edit'>('new')
const currentSkin = ref<Skin | null>(null)
const shouldRestoreModal = ref(false)
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
	shouldRestoreModal.value = false
	isSaving.value = false
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
	nextTick(() => forceCapeScrollCheck())
}

async function showNew(e: MouseEvent, skinTextureUrl: SkinTextureUrl) {
	mode.value = 'new'
	currentSkin.value = null
	uploadedTextureUrl.value = skinTextureUrl
	variant.value = await determineModelType(skinTextureUrl.original)
	selectedCape.value = undefined

	await loadPreviewSkin()

	modal.value?.show(e)
	nextTick(() => forceCapeScrollCheck())
}

async function restoreWithNewTexture(skinTextureUrl: SkinTextureUrl) {
	uploadedTextureUrl.value = skinTextureUrl
	await loadPreviewSkin()

	if (shouldRestoreModal.value) {
		setTimeout(() => {
			modal.value?.show()
			nextTick(() => forceCapeScrollCheck())
			shouldRestoreModal.value = false
		}, 0)
	}
}

function hide() {
	modal.value?.hide()
	setTimeout(() => resetState(), 250)
}

function selectCape(cape: Cape | undefined) {
	selectedCape.value = cape
}

function openUploadSkinModal(e: MouseEvent) {
	shouldRestoreModal.value = true
	modal.value?.hide()
	emit('open-upload-modal', e)
}

function restoreModal() {
	if (shouldRestoreModal.value) {
		setTimeout(() => {
			const fakeEvent = new MouseEvent('click')
			modal.value?.show(fakeEvent)
			nextTick(() => forceCapeScrollCheck())
			shouldRestoreModal.value = false
		}, 500)
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
})

watch(
	() => props.capes,
	() => {
		nextTick(() => forceCapeScrollCheck())
	},
	{ immediate: true },
)

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
	shouldRestoreModal,
	restoreModal,
})
</script>
