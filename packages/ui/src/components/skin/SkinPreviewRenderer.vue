<template>
	<!-- eslint-disable vue/no-undef-components -->
	<div
		ref="skinPreviewContainer"
		class="relative w-full h-full overflow-visible cursor-grab"
		@click="onCanvasClick"
	>
		<div
			class="absolute left-0 right-0 z-10 flex items-center justify-center pointer-events-none"
			:style="previewControlsPositionStyle"
		>
			<span
				class="flex items-center justify-center gap-1.5 text-base font-medium leading-6 text-primary"
			>
				<UnfoldHorizontalIcon class="size-5 shrink-0" />
				Drag to rotate
			</span>
		</div>
		<div
			v-if="$slots.subtitle"
			class="absolute left-0 right-0 z-10 flex items-center justify-center pointer-events-none"
			:style="subtitlePositionStyle"
		>
			<div ref="subtitleElement" class="w-full pointer-events-auto" @click="ignoreControlClick">
				<slot name="subtitle" />
			</div>
		</div>
		<div
			v-if="nametag || $slots['nametag-badge']"
			class="absolute left-1/2 pointer-events-none z-10"
			:style="nametagStyle"
		>
			<div
				v-if="$slots['nametag-badge']"
				class="absolute bottom-[calc(100%+1rem)] left-1/2 flex -translate-x-1/2 items-center justify-center"
			>
				<slot name="nametag-badge" />
			</div>
			<div v-if="nametag" class="px-3 py-1 rounded-md font-minecraft text-gray nametag-bg">
				{{ nametagText }}
			</div>
		</div>

		<TresCanvas
			alpha
			:antialias="true"
			:dpr="rendererDpr"
			:renderer-options="{
				outputColorSpace: THREE.SRGBColorSpace,
				toneMapping: THREE.NoToneMapping,
				toneMappingExposure: 10.0,
			}"
			class="transition-opacity duration-500"
			:class="{ 'opacity-0': !isPreviewVisible, 'opacity-100': isPreviewVisible }"
			@pointerdown="onPointerDown"
			@pointermove="onPointerMove"
			@pointerup="onPointerUp"
			@pointerleave="onPointerUp"
		>
			<Suspense>
				<Group
					:rotation="animatedModelGroupRotation"
					:position="animatedModelGroupPosition"
					:scale="animatedModelGroupScale"
				>
					<Group :position="modelOffset">
						<primitive v-if="scene" :object="scene" />
					</Group>
				</Group>
			</Suspense>

			<Suspense>
				<TresMesh
					:position="spotlightPosition"
					:rotation="[-Math.PI / 2, 0, 0]"
					:scale="spotlightScale"
				>
					<TresCircleGeometry :args="[1, 128]" />
					<TresShaderMaterial v-bind="radialSpotlightShader" />
				</TresMesh>
			</Suspense>

			<TresPerspectiveCamera
				:make-default.camel="true"
				:fov="cameraConfig.fov"
				:position="cameraConfig.position"
				:look-at="cameraConfig.target"
			/>

			<TresAmbientLight :intensity="2" />
			<TresDirectionalLight :position="[-3, 4, -2]" :intensity="1.2" />
		</TresCanvas>

		<div v-if="showLoading" class="absolute inset-0 flex items-center justify-center">
			<div class="text-primary">Loading...</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ClassicPlayerModel, SlimPlayerModel, UnfoldHorizontalIcon } from '@modrinth/assets'
import { TresCanvas } from '@tresjs/core'
import * as THREE from 'three'
import {
	computed,
	nextTick,
	onMounted,
	onUnmounted,
	ref,
	toRef,
	useSlots,
	useTemplateRef,
	watch,
} from 'vue'

import type {
	SkinPreviewAnimationConfig,
	SkinPreviewFitPadding,
	SkinPreviewFraming,
	SkinPreviewTuple,
} from '#ui/composables/skin-rendering'
import {
	useSkinPreviewAnimation,
	useSkinPreviewControls,
	useSkinPreviewFit,
	useSkinPreviewLoading,
	useSkinPreviewScene,
} from '#ui/composables/skin-rendering'

import { useDynamicFontSize } from '../../composables'
import { createRadialSpotlightShader, syncDamageFlashShader } from './skin-preview-shader'

const props = withDefaults(
	defineProps<{
		textureSrc: string
		earsTextureSrc?: string
		capeSrc?: string
		variant?: 'SLIM' | 'CLASSIC' | 'UNKNOWN'
		nametag?: string
		fit?: boolean
		lockFit?: boolean
		framing?: SkinPreviewFraming
		fitZoom?: number
		fitPadding?: Partial<SkinPreviewFitPadding>
		/** @deprecated Manual framing fallback. */
		scale?: number
		/** @deprecated Manual framing fallback, or auto-fit FOV override when fit=true. */
		fov?: number
		initialRotation?: number
		animationConfig?: SkinPreviewAnimationConfig
		earsEnabled?: boolean
	}>(),
	{
		variant: 'CLASSIC',
		earsTextureSrc: undefined,
		capeSrc: undefined,
		initialRotation: 15.75,
		nametag: undefined,
		fit: undefined,
		lockFit: true,
		framing: 'page',
		fitZoom: 1,
		earsEnabled: true,
		animationConfig: () => ({
			baseAnimation: 'idle',
			randomAnimations: ['idle_sub_1', 'idle_sub_2', 'idle_sub_3'],
			randomAnimationInterval: 8000,
			transitionDuration: 0.2,
		}),
	},
)

const emit = defineEmits<{
	earsFeaturesDetected: [detected: boolean]
}>()

const skinPreviewContainer = useTemplateRef<HTMLElement>('skinPreviewContainer')
const subtitleElement = useTemplateRef<HTMLElement>('subtitleElement')
const slots = useSlots()
const nametagText = computed(() => props.nametag)
const hasSubtitle = computed(() => Boolean(slots.subtitle))
const hasNametagBadge = computed(() => Boolean(slots['nametag-badge']))
const isSubtitleWrapped = ref(false)
const selectedModelSrc = computed(() =>
	props.variant === 'SLIM' ? SlimPlayerModel : ClassicPlayerModel,
)

let subtitleResizeObserver: ResizeObserver | undefined

function getSubtitleLayoutRoot(element: HTMLElement) {
	const elementChildren = Array.from(element.children).filter(
		(child): child is HTMLElement => child instanceof HTMLElement,
	)

	return elementChildren.length === 1 ? elementChildren[0] : element
}

function updateSubtitleWrapped() {
	const element = subtitleElement.value
	if (!element) {
		isSubtitleWrapped.value = false
		return
	}

	const layoutRoot = getSubtitleLayoutRoot(element)
	const children = Array.from(layoutRoot.children).filter(
		(child): child is HTMLElement => child instanceof HTMLElement,
	)

	if (children.length < 2) {
		isSubtitleWrapped.value = false
		return
	}

	const firstTop = children[0].getBoundingClientRect().top
	isSubtitleWrapped.value = children.some(
		(child) => Math.abs(child.getBoundingClientRect().top - firstTop) > 1,
	)
}

function observeSubtitleElement() {
	subtitleResizeObserver?.disconnect()

	const element = subtitleElement.value
	if (!element) {
		isSubtitleWrapped.value = false
		return
	}

	const layoutRoot = getSubtitleLayoutRoot(element)

	subtitleResizeObserver = new ResizeObserver(updateSubtitleWrapped)
	subtitleResizeObserver.observe(element)
	if (layoutRoot !== element) {
		subtitleResizeObserver.observe(layoutRoot)
	}

	void nextTick(updateSubtitleWrapped)
}

const {
	cleanupAnimationState,
	clickImpulseOffsetX,
	clickImpulseRotationZ,
	clickImpulseScaleX,
	clickImpulseScaleY,
	currentAnimation,
	damageFlashIntensity,
	getAvailableAnimations,
	initializeAnimations,
	playAnimation,
	playClickInteraction,
	stopAnimations,
} = useSkinPreviewAnimation(toRef(props, 'animationConfig'))

const {
	ignoreControlClick,
	modelRotation,
	onCanvasClick,
	onPointerDown,
	onPointerMove,
	onPointerUp,
} = useSkinPreviewControls({
	initialRotation: toRef(props, 'initialRotation'),
	onClickWithoutDrag: () => {
		playClickInteraction()
	},
})

const { hasEarsFeatures, isModelLoaded, isTextureLoaded, modelCenter, modelSize, scene } =
	useSkinPreviewScene({
		selectedModelSrc,
		textureSrc: toRef(props, 'textureSrc'),
		earsTextureSrc: toRef(props, 'earsTextureSrc'),
		capeSrc: toRef(props, 'capeSrc'),
		earsEnabled: toRef(props, 'earsEnabled'),
		initializeAnimations,
		cleanupAnimationState,
	})

function syncDamageFlashShaderMaterials() {
	syncDamageFlashShader(scene.value, damageFlashIntensity.value)
}

const {
	cameraConfig,
	fitEnabled,
	hasResolvedFit,
	modelGroupPosition,
	modelGroupScale,
	modelOffset,
	nametagTop,
	previewControlsPositionStyle,
	spotlightPosition,
	spotlightScale,
	subtitlePositionStyle,
} = useSkinPreviewFit({
	containerElement: computed(() => skinPreviewContainer.value),
	fit: toRef(props, 'fit'),
	lockFit: toRef(props, 'lockFit'),
	framing: toRef(props, 'framing'),
	fitZoom: toRef(props, 'fitZoom'),
	fitPadding: toRef(props, 'fitPadding'),
	scale: toRef(props, 'scale'),
	fov: toRef(props, 'fov'),
	modelRotation,
	nametag: toRef(props, 'nametag'),
	hasSubtitle,
	hasNametagBadge,
	subtitleWrapped: isSubtitleWrapped,
	modelCenter,
	modelSize,
	isModelLoaded,
})

const rendererDpr: [number, number] = [1, 1.5]
const radialSpotlightShader = createRadialSpotlightShader()
const isReady = computed(() => isModelLoaded.value && isTextureLoaded.value && hasResolvedFit.value)
const { isPreviewVisible, showLoading } = useSkinPreviewLoading(isReady)

onMounted(observeSubtitleElement)

watch(hasSubtitle, () => nextTick(observeSubtitleElement), { flush: 'post' })
watch(
	hasEarsFeatures,
	(detected) => {
		emit('earsFeaturesDetected', detected)
	},
	{ immediate: true },
)
watch(scene, syncDamageFlashShaderMaterials, { immediate: true })
watch(damageFlashIntensity, syncDamageFlashShaderMaterials)

onUnmounted(() => {
	subtitleResizeObserver?.disconnect()
})

const { fontSize: nametagFontSize } = useDynamicFontSize({
	containerElement: skinPreviewContainer,
	text: nametagText,
	baseFontSize: 1.8,
	minFontSize: 1.25,
	maxFontSize: 2,
	padding: 24,
	fontFamily: 'inherit',
})

const nametagStyle = computed(() => ({
	fontSize: nametagFontSize.value,
	top: nametagTop.value,
	transform: fitEnabled.value ? 'translate(-50%, calc(-100% - 0.75rem))' : 'translateX(-50%)',
}))

const animatedModelGroupRotation = computed<SkinPreviewTuple>(() => [
	0,
	modelRotation.value,
	clickImpulseRotationZ.value,
])

const animatedModelGroupPosition = computed<SkinPreviewTuple>(() => {
	const [x, y, z] = modelGroupPosition.value
	return [x + clickImpulseOffsetX.value, y, z]
})

const animatedModelGroupScale = computed<SkinPreviewTuple>(() => {
	const [x, y, z] = modelGroupScale.value
	return [x * clickImpulseScaleX.value, y * clickImpulseScaleY.value, z]
})

defineExpose({
	playAnimation,
	playClickInteraction,
	stopAnimations,
	getAvailableAnimations,
	getCurrentAnimation: () => currentAnimation.value,
})
</script>

<style scoped lang="scss">
.nametag-bg {
	background:
		linear-gradient(308.68deg, rgba(50, 50, 50, 0.2) -52.46%, rgba(100, 100, 100, 0.2) 94.75%),
		rgba(0, 0, 0, 0.2);
	box-shadow:
		inset -0.5px -0.5px 0px rgba(0, 0, 0, 0.25),
		inset 0.5px 0.5px 0px rgba(255, 255, 255, 0.05);
}
</style>
