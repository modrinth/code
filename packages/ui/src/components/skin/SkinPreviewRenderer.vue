<template>
	<!-- eslint-disable vue/no-undef-components -->
	<div
		ref="skinPreviewContainer"
		class="relative w-full h-full overflow-hidden cursor-grab"
		@click="onCanvasClick"
	>
		<div
			class="absolute left-0 right-0 z-10 flex items-center justify-center pointer-events-none"
			:class="previewControlsPositionClass"
		>
			<span
				class="flex items-center justify-center gap-1.5 text-base font-medium leading-6 text-primary"
			>
				<ArrowLeftRightIcon class="size-5 shrink-0" />
				Drag to rotate
			</span>
		</div>
		<div
			v-if="$slots.subtitle"
			class="absolute left-0 right-0 z-10 flex items-center justify-center pointer-events-none"
			:class="subtitlePositionClass"
		>
			<div class="pointer-events-auto">
				<slot name="subtitle" />
			</div>
		</div>
		<div
			v-if="nametag"
			class="absolute left-1/2 px-3 py-1 rounded-md pointer-events-none z-10 font-minecraft text-gray nametag-bg transition-all duration-200"
			:style="nametagStyle"
		>
			{{ nametagText }}
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
			:class="{ 'opacity-0': !isReady, 'opacity-100': isReady }"
			@pointerdown="onPointerDown"
			@pointermove="onPointerMove"
			@pointerup="onPointerUp"
			@pointerleave="onPointerUp"
		>
			<Suspense>
				<Group
					:rotation="[0, modelRotation, 0]"
					:position="modelGroupPosition"
					:scale="modelGroupScale"
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

		<div
			v-if="!isReady"
			class="absolute inset-0 flex items-center justify-center transition-opacity duration-500"
			:class="{ 'opacity-100': !isReady, 'opacity-0': isReady }"
		>
			<div class="text-primary">Loading...</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ArrowLeftRightIcon, ClassicPlayerModel, SlimPlayerModel } from '@modrinth/assets'
import {
	applyCapeTexture,
	applyTexture,
	createTransparentTexture,
	loadTexture as loadSkinTexture,
} from '@modrinth/utils'
import { useGLTF } from '@tresjs/cientos'
import { TresCanvas, useRenderLoop, useTexture } from '@tresjs/core'
import * as THREE from 'three'
import { clone as cloneSkeleton } from 'three/examples/jsm/utils/SkeletonUtils.js'
import {
	computed,
	markRaw,
	onBeforeMount,
	onMounted,
	onUnmounted,
	ref,
	shallowRef,
	toRefs,
	useSlots,
	useTemplateRef,
	watch,
} from 'vue'

import { useDynamicFontSize } from '../../composables'

interface AnimationConfig {
	baseAnimation: string
	randomAnimations: string[]
	randomAnimationInterval?: number
	transitionDuration?: number
}

type SkinPreviewFraming = 'page' | 'modal'

interface FitPadding {
	top: number
	right: number
	bottom: number
	left: number
}

interface FitLock {
	containerSize: {
		width: number
		height: number
	}
	modelCenter: [number, number, number]
	modelSize: [number, number, number]
}

type AnimationFinishedListener = (
	event: THREE.AnimationMixerEventMap['finished'] & {
		readonly type: 'finished'
		readonly target: THREE.AnimationMixer
	},
) => void

const props = withDefaults(
	defineProps<{
		textureSrc: string
		capeSrc?: string
		variant?: 'SLIM' | 'CLASSIC' | 'UNKNOWN'
		nametag?: string
		fit?: boolean
		lockFit?: boolean
		framing?: SkinPreviewFraming
		fitZoom?: number
		fitPadding?: Partial<FitPadding>
		/** @deprecated Manual framing fallback. */
		scale?: number
		/** @deprecated Manual framing fallback, or auto-fit FOV override when fit=true. */
		fov?: number
		initialRotation?: number
		animationConfig?: AnimationConfig
	}>(),
	{
		variant: 'CLASSIC',
		capeSrc: undefined,
		initialRotation: 15.75,
		nametag: undefined,
		fit: undefined,
		lockFit: true,
		framing: 'page',
		fitZoom: 1,
		animationConfig: () => ({
			baseAnimation: 'idle',
			randomAnimations: ['idle_sub_1', 'idle_sub_2', 'idle_sub_3'],
			randomAnimationInterval: 8000,
			transitionDuration: 0.2,
		}),
	},
)

const skinPreviewContainer = useTemplateRef<HTMLElement>('skinPreviewContainer')
const slots = useSlots()
const nametagText = computed(() => props.nametag)

const fitEnabled = computed(() => {
	if (props.fit !== undefined) return props.fit
	return props.scale === undefined && props.fov === undefined
})
const currentFraming = computed<SkinPreviewFraming>(() => props.framing ?? 'page')
const legacyScale = computed(() => props.scale ?? 1)
const legacyFov = computed(() => props.fov ?? 40)
const previewControlsPositionClass = computed(() =>
	currentFraming.value === 'modal' ? 'bottom-[calc(6%)]' : 'bottom-[calc(15%+64px)]',
)
const subtitlePositionClass = computed(() =>
	currentFraming.value === 'modal' ? 'bottom-[6%]' : 'bottom-[calc(15%)]',
)

const { fontSize: nametagFontSize } = useDynamicFontSize({
	containerElement: skinPreviewContainer,
	text: nametagText,
	baseFontSize: 1.8,
	minFontSize: 1.25,
	maxFontSize: 2,
	padding: 24,
	fontFamily: 'inherit',
})

const containerSize = ref({ width: 1, height: 1 })
let resizeObserver: ResizeObserver | undefined

onMounted(() => {
	const el = skinPreviewContainer.value
	if (!el) return

	resizeObserver = new ResizeObserver(([entry]) => {
		const { width, height } = entry.contentRect
		containerSize.value = {
			width: Math.max(width, 1),
			height: Math.max(height, 1),
		}

		if (props.lockFit) {
			lockFitState()
		}
	})

	resizeObserver.observe(el)
})

const selectedModelSrc = computed(() =>
	props.variant === 'SLIM' ? SlimPlayerModel : ClassicPlayerModel,
)

const scene = shallowRef<THREE.Object3D | null>(null)
const lastCapeSrc = ref<string | undefined>(undefined)
const texture = shallowRef<THREE.Texture | null>(null)
const capeTexture = shallowRef<THREE.Texture | null>(null)
const transparentTexture = createTransparentTexture()
const rendererDpr: [number, number] = [1, 1.5]
const modelCenter = ref<[number, number, number]>([0, 1, 0])
const modelSize = ref<[number, number, number]>([1, 2, 1])

const isModelLoaded = ref(false)
const isTextureLoaded = ref(false)
const fitLock = ref<FitLock | null>(null)
const hasUsableFitSize = computed(
	() => containerSize.value.width > 1 && containerSize.value.height > 1,
)
const hasResolvedFit = computed(
	() => !fitEnabled.value || (props.lockFit ? fitLock.value !== null : hasUsableFitSize.value),
)
const isReady = computed(() => isModelLoaded.value && isTextureLoaded.value && hasResolvedFit.value)

const fitContainerSize = computed(() =>
	props.lockFit ? (fitLock.value?.containerSize ?? containerSize.value) : containerSize.value,
)
const fitModelCenter = computed(() =>
	props.lockFit ? (fitLock.value?.modelCenter ?? modelCenter.value) : modelCenter.value,
)
const fitModelSize = computed(() =>
	props.lockFit ? (fitLock.value?.modelSize ?? modelSize.value) : modelSize.value,
)

const mixer = ref<THREE.AnimationMixer | null>(null)
const actions = ref<Record<string, THREE.AnimationAction>>({})
const clock = new THREE.Clock()
const currentAnimation = ref<string>('')
const randomAnimationTimer = ref<number | null>(null)
const lastRandomAnimation = ref<string>('')
const animationFinishedListeners: AnimationFinishedListener[] = []
let modelLoadVersion = 0
let isUnmounted = false

const radialSpotlightShader = computed(() => ({
	uniforms: {
		innerColor: { value: new THREE.Color(0x000000) },
		outerColor: { value: new THREE.Color(0xffffff) },
		innerOpacity: { value: 0.3 },
		outerOpacity: { value: 0.0 },
		falloffPower: { value: 1.2 },
		shadowRadius: { value: 7 },
	},
	vertexShader: `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
	fragmentShader: `
    uniform vec3 innerColor;
    uniform vec3 outerColor;
    uniform float innerOpacity;
    uniform float outerOpacity;
    uniform float falloffPower;
    uniform float shadowRadius;

    varying vec2 vUv;

    void main() {
      vec2 center = vec2(0.5, 0.5);
      float dist = distance(vUv, center) * 2.0;

      // Create shadow in the center
      float shadowFalloff = 1.0 - smoothstep(0.0, shadowRadius, dist);

      // Create overall spotlight falloff
      float spotlightFalloff = 1.0 - smoothstep(0.0, 1.0, pow(dist, falloffPower));

      // Combine both effects
      vec3 color = mix(outerColor, innerColor, shadowFalloff);
      float opacity = mix(outerOpacity, innerOpacity * shadowFalloff, spotlightFalloff);

      gl_FragColor = vec4(color, opacity);
    }
  `,
	transparent: true,
	depthWrite: false,
	depthTest: false,
}))

const FRAMING_PRESETS = {
	page: {
		fov: 35,
		zoom: 0.96,
		padding: { top: 0.2, right: 0.14, bottom: 0.3, left: 0.14 },
	},
	modal: {
		fov: 35,
		zoom: 1,
		padding: { top: 0.1, right: 0.1, bottom: 0.18, left: 0.1 },
	},
} satisfies Record<SkinPreviewFraming, { fov: number; zoom: number; padding: FitPadding }>

const resolvedFitPadding = computed<FitPadding>(() => {
	const preset = FRAMING_PRESETS[currentFraming.value].padding

	return {
		top: Math.max(preset.top, props.nametag ? 0.2 : 0),
		right: preset.right,
		bottom: Math.max(preset.bottom, slots.subtitle ? 0.28 : preset.bottom),
		left: preset.left,
		...(props.fitPadding ?? {}),
	}
})

const modelOffset = computed<[number, number, number]>(() => {
	if (!fitEnabled.value) return [0, 0, 0]

	const [x, y, z] = fitModelCenter.value
	return [-x, -y, -z]
})

const modelGroupPosition = computed<[number, number, number]>(() => {
	if (fitEnabled.value) return [0, 0, 0]
	return [0, -0.05 * legacyScale.value, 1.95]
})

const modelGroupScale = computed<[number, number, number]>(() => {
	if (fitEnabled.value) return [1, 1, 1]

	const scale = 0.8 * legacyScale.value
	return [scale, scale, scale]
})

const fittedCamera = computed(() => {
	const width = Math.max(fitContainerSize.value.width, 1)
	const height = Math.max(fitContainerSize.value.height, 1)
	const aspect = width / height
	const preset = FRAMING_PRESETS[currentFraming.value]
	const padding = resolvedFitPadding.value

	const usableWidth = Math.max(width * (1 - padding.left - padding.right), 1)
	const usableHeight = Math.max(height * (1 - padding.top - padding.bottom), 1)

	const [sizeX, sizeY, sizeZ] = fitModelSize.value
	const halfWidth = Math.sqrt((sizeX / 2) ** 2 + (sizeZ / 2) ** 2)
	const halfHeight = sizeY / 2

	const fov = props.fov ?? preset.fov
	const verticalFov = THREE.MathUtils.degToRad(fov)
	const horizontalFov = 2 * Math.atan(Math.tan(verticalFov / 2) * aspect)

	const paddedHalfWidth = halfWidth * (width / usableWidth)
	const paddedHalfHeight = halfHeight * (height / usableHeight)
	const zoom = Math.max((props.fitZoom ?? 1) * preset.zoom, 0.01)

	const distance =
		Math.max(
			paddedHalfHeight / Math.tan(verticalFov / 2),
			paddedHalfWidth / Math.tan(horizontalFov / 2),
		) / zoom

	const visibleHalfHeight = distance * Math.tan(verticalFov / 2)
	const targetY = -(padding.bottom - padding.top) * visibleHalfHeight

	return {
		fov,
		position: [0, targetY, -distance] as [number, number, number],
		target: [0, targetY, 0] as [number, number, number],
	}
})

const cameraConfig = computed(() => {
	if (fitEnabled.value) return fittedCamera.value

	return {
		fov: legacyFov.value,
		position: [0, 1.5, -3.25] as [number, number, number],
		target: modelCenter.value,
	}
})

const nametagTop = computed(() => {
	if (!fitEnabled.value) return '18%'

	const [, sizeY] = fitModelSize.value
	const { fov, position, target } = cameraConfig.value
	const distance = Math.max(Math.abs(position[2] - target[2]), 0.001)
	const verticalFov = THREE.MathUtils.degToRad(fov)
	const modelTopY = sizeY / 2
	const projectedY = (modelTopY - target[1]) / distance / Math.max(Math.tan(verticalFov / 2), 0.001)
	const topPercent = THREE.MathUtils.clamp(((1 - projectedY) / 2) * 100, 8, 40)

	return `${topPercent - 2}%`
})

const nametagStyle = computed(() => ({
	fontSize: nametagFontSize.value,
	top: nametagTop.value,
	transform: fitEnabled.value ? 'translate(-50%, calc(-100% - 0.75rem))' : 'translateX(-50%)',
}))

const spotlightY = computed(() => {
	if (!fitEnabled.value) return -0.1 * legacyScale.value

	const [, sizeY] = fitModelSize.value
	return -sizeY / 2 - 0.02
})

const spotlightPosition = computed<[number, number, number]>(() => [
	0,
	spotlightY.value,
	fitEnabled.value ? 0 : 2,
])

const spotlightScale = computed<[number, number, number]>(() => {
	if (!fitEnabled.value) {
		const scale = 0.75 * legacyScale.value
		return [scale, scale, scale]
	}

	const [sizeX, , sizeZ] = fitModelSize.value
	const radius = Math.max(sizeX, sizeZ, 1) * 0.8
	return [radius, radius, radius]
})

const { baseAnimation, randomAnimations } = toRefs(props.animationConfig)

function initializeAnimations(loadedScene: THREE.Object3D, clips: THREE.AnimationClip[]) {
	if (!clips || clips.length === 0) {
		console.warn('No animation clips found in the model')
		return
	}

	mixer.value = new THREE.AnimationMixer(loadedScene)
	actions.value = {}

	clips.forEach((clip) => {
		const action = mixer.value!.clipAction(clip)

		action.setLoop(THREE.LoopOnce, 1)
		action.clampWhenFinished = true
		actions.value[clip.name] = action
	})

	if (baseAnimation.value && actions.value[baseAnimation.value]) {
		actions.value[baseAnimation.value].setLoop(THREE.LoopRepeat, Infinity)
		playAnimation(baseAnimation.value)
		setupRandomAnimationLoop()
	} else {
		console.warn(`Base animation "${baseAnimation.value}" not found`)

		const firstAnimationName = Object.keys(actions.value)[0]
		if (firstAnimationName) {
			actions.value[firstAnimationName].setLoop(THREE.LoopRepeat, Infinity)
			playAnimation(firstAnimationName)
		}
	}
}

function playAnimation(name: string) {
	if (!mixer.value || !actions.value[name]) {
		console.warn(`Animation "${name}" not found!`)
		return false
	}

	const action = actions.value[name]

	if (currentAnimation.value === name && action.isRunning() && name !== baseAnimation.value) {
		console.log(`Animation "${name}" is already running, ignoring request`)
		return false
	}

	const transitionDuration = props.animationConfig.transitionDuration || 0.3

	Object.entries(actions.value).forEach(([actionName, actionInstance]) => {
		if (actionName !== name && actionInstance.isRunning()) {
			actionInstance.fadeOut(transitionDuration)
		}
	})

	action.reset()

	if (name === baseAnimation.value) {
		action.setLoop(THREE.LoopRepeat, Infinity)
	} else {
		action.setLoop(THREE.LoopOnce, 1)
		action.clampWhenFinished = true

		const onFinished: AnimationFinishedListener = (event) => {
			if (event.action === action) {
				removeAnimationFinishedListener(onFinished)
				if (currentAnimation.value === name && baseAnimation.value) {
					action.fadeOut(transitionDuration)
					const baseAction = actions.value[baseAnimation.value]
					baseAction.reset()
					baseAction.fadeIn(transitionDuration)
					baseAction.play()
					currentAnimation.value = baseAnimation.value
				}
			}
		}

		addAnimationFinishedListener(onFinished)
	}

	action.fadeIn(transitionDuration)
	action.play()

	currentAnimation.value = name
	return true
}

function setupRandomAnimationLoop() {
	const interval = props.animationConfig.randomAnimationInterval || 10000

	function scheduleNextAnimation() {
		if (randomAnimationTimer.value) {
			clearTimeout(randomAnimationTimer.value)
		}

		randomAnimationTimer.value = window.setTimeout(() => {
			if (randomAnimations.value.length > 0 && currentAnimation.value === baseAnimation.value) {
				const availableAnimations = randomAnimations.value.filter(
					(anim) => anim !== lastRandomAnimation.value,
				)

				// If all animations have been used, reset and use the full list
				const animationsToChooseFrom =
					availableAnimations.length > 0 ? availableAnimations : randomAnimations.value

				const randomIndex = Math.floor(Math.random() * animationsToChooseFrom.length)
				const randomAnimationName = animationsToChooseFrom[randomIndex]

				if (actions.value[randomAnimationName]) {
					lastRandomAnimation.value = randomAnimationName
					playRandomAnimation(randomAnimationName)
				}
			} else {
				// If not in base animation, wait and try again
				scheduleNextAnimation()
			}
		}, interval)
	}

	scheduleNextAnimation()
}

function playRandomAnimation(name: string) {
	if (!mixer.value || !actions.value[name]) {
		console.warn(`Animation "${name}" not found!`)
		return
	}

	const action = actions.value[name]

	if (currentAnimation.value === name && action.isRunning()) {
		console.log(`Animation "${name}" is already running, ignoring request`)
		return
	}

	const transitionDuration = props.animationConfig.transitionDuration || 0.3

	if (baseAnimation.value && actions.value[baseAnimation.value].isRunning()) {
		actions.value[baseAnimation.value].fadeOut(transitionDuration)
	}

	action.reset()
	action.setLoop(THREE.LoopOnce, 1)
	action.clampWhenFinished = true
	action.fadeIn(transitionDuration)
	action.play()

	currentAnimation.value = name

	const onFinished: AnimationFinishedListener = (event) => {
		if (event.action === action) {
			removeAnimationFinishedListener(onFinished)
			if (currentAnimation.value === name && baseAnimation.value) {
				action.fadeOut(transitionDuration)
				const baseAction = actions.value[baseAnimation.value]
				baseAction.reset()
				baseAction.fadeIn(transitionDuration)
				baseAction.play()
				currentAnimation.value = baseAnimation.value

				// Schedule the next random animation after returning to base
				setupRandomAnimationLoop()
			}
		}
	}

	addAnimationFinishedListener(onFinished)
}

function stopAnimations() {
	if (mixer.value) {
		mixer.value.stopAllAction()
	}
	currentAnimation.value = ''
}

function getAvailableAnimations(): string[] {
	return Object.keys(actions.value)
}

function clearRandomAnimationTimer() {
	if (randomAnimationTimer.value) {
		clearTimeout(randomAnimationTimer.value)
		randomAnimationTimer.value = null
	}
}

function addAnimationFinishedListener(listener: AnimationFinishedListener) {
	mixer.value?.addEventListener('finished', listener)
	animationFinishedListeners.push(listener)
}

function removeAnimationFinishedListener(
	listener: AnimationFinishedListener,
	targetMixer = mixer.value,
) {
	targetMixer?.removeEventListener('finished', listener)

	const index = animationFinishedListeners.indexOf(listener)
	if (index !== -1) {
		animationFinishedListeners.splice(index, 1)
	}
}

function clearAnimationFinishedListeners(targetMixer = mixer.value) {
	animationFinishedListeners.forEach((listener) => {
		targetMixer?.removeEventListener('finished', listener)
	})
	animationFinishedListeners.length = 0
}

function cleanupAnimationState(root: THREE.Object3D | null) {
	clearRandomAnimationTimer()

	const currentMixer = mixer.value
	if (currentMixer) {
		clearAnimationFinishedListeners(currentMixer)
		currentMixer.stopAllAction()

		if (root) {
			currentMixer.uncacheRoot(root)
		}
	}

	mixer.value = null
	actions.value = {}
	currentAnimation.value = ''
	lastRandomAnimation.value = ''
}

function disposeSceneMaterials(root: THREE.Object3D | null) {
	if (!root) return

	const materials = new Set<THREE.Material>()

	root.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (!mesh.isMesh || !mesh.material) return

		const meshMaterials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		meshMaterials.forEach((material) => materials.add(material))
	})

	materials.forEach((material) => material.dispose())
}

defineExpose({
	playAnimation,
	stopAnimations,
	getAvailableAnimations,
	getCurrentAnimation: () => currentAnimation.value,
})

const { onLoop } = useRenderLoop()
onLoop(() => {
	if (mixer.value) {
		mixer.value.update(clock.getDelta())
	}
})

const SKIN_LAYER_DEPTH_OFFSET: Record<string, number> = {
	Body_Layer: -4,
	Left_Leg_Layer: -8,
	Right_Leg_Layer: -8,
	Left_Arm_Layer: -12,
	Right_Arm_Layer: -12,
	Hat_Layer: -16,
}

function configureSkinPreviewMesh(mesh: THREE.Mesh) {
	const isSkinLayer = mesh.name.endsWith('_Layer')
	const layerDepthOffset = SKIN_LAYER_DEPTH_OFFSET[mesh.name] ?? 0
	mesh.renderOrder = isSkinLayer ? Math.abs(layerDepthOffset) : 0

	const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
	materials.forEach((material) => {
		if (!(material instanceof THREE.MeshStandardMaterial) || material.name === 'cape') return

		material.transparent = false
		material.alphaTest = 0.1
		material.depthTest = true
		material.depthWrite = true
		material.polygonOffset = isSkinLayer
		material.polygonOffsetFactor = layerDepthOffset
		material.polygonOffsetUnits = layerDepthOffset
		material.needsUpdate = true
	})
}

function cloneSceneForRenderer(source: THREE.Object3D) {
	const cloned = cloneSkeleton(source)

	cloned.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (!mesh.isMesh || !mesh.material) return

		mesh.material = Array.isArray(mesh.material)
			? mesh.material.map((material) => material.clone())
			: mesh.material.clone()

		configureSkinPreviewMesh(mesh)
	})

	return markRaw(cloned)
}

function cloneModelTuple(tuple: [number, number, number]): [number, number, number] {
	return [tuple[0], tuple[1], tuple[2]]
}

function lockFitState() {
	if (!fitEnabled.value || !props.lockFit || fitLock.value || !isModelLoaded.value) return

	const { width, height } = containerSize.value
	if (width <= 1 || height <= 1) return

	fitLock.value = {
		containerSize: { width, height },
		modelCenter: cloneModelTuple(modelCenter.value),
		modelSize: cloneModelTuple(modelSize.value),
	}
}

async function loadModel(src: string) {
	const loadVersion = ++modelLoadVersion

	try {
		isModelLoaded.value = false
		fitLock.value = null
		const { scene: loadedScene, animations } = await useGLTF(src)
		const clonedScene = cloneSceneForRenderer(loadedScene)
		if (isUnmounted || loadVersion !== modelLoadVersion) {
			disposeSceneMaterials(clonedScene)
			return
		}

		const previousScene = scene.value
		cleanupAnimationState(previousScene)
		disposeSceneMaterials(previousScene)
		scene.value = clonedScene

		if (texture.value) {
			applyTexture(clonedScene, texture.value)
		}

		applyCapeTexture(clonedScene, capeTexture.value, transparentTexture)

		if (animations && animations.length > 0) {
			initializeAnimations(clonedScene, animations)
		}

		updateModelInfo()
		isModelLoaded.value = true
		lockFitState()
	} catch (error) {
		console.error('Failed to load model:', error)
		if (!isUnmounted && loadVersion === modelLoadVersion) {
			isModelLoaded.value = false
		}
	}
}

async function loadAndApplyTexture(src: string) {
	if (!src) return null

	try {
		try {
			return await loadSkinTexture(src)
		} catch {
			const tex = await useTexture([src])
			tex.colorSpace = THREE.SRGBColorSpace
			tex.flipY = false
			tex.magFilter = THREE.NearestFilter
			tex.minFilter = THREE.NearestFilter
			return tex
		}
	} catch (error) {
		console.error('Failed to load texture:', error)
		return null
	}
}

async function loadAndApplyCapeTexture(src: string | undefined) {
	if (src === lastCapeSrc.value) return

	lastCapeSrc.value = src

	if (src) {
		capeTexture.value = await loadAndApplyTexture(src)
	} else {
		capeTexture.value = null
	}

	if (scene.value) {
		applyCapeTexture(scene.value, capeTexture.value, transparentTexture)
	}
}

function getVisibleMeshBox(root: THREE.Object3D): THREE.Box3 | null {
	root.updateWorldMatrix(true, true)

	const result = new THREE.Box3()
	const meshBox = new THREE.Box3()
	let found = false

	root.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (!mesh.isMesh || !mesh.geometry || mesh.visible === false) return

		const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		if (materials.length && materials.every((material) => material.visible === false)) return

		if (!mesh.geometry.boundingBox) {
			mesh.geometry.computeBoundingBox()
		}

		if (!mesh.geometry.boundingBox) return

		meshBox.copy(mesh.geometry.boundingBox).applyMatrix4(mesh.matrixWorld)
		result.union(meshBox)
		found = true
	})

	return found && !result.isEmpty() ? result.clone() : null
}

function updateModelInfo() {
	const box = scene.value ? getVisibleMeshBox(scene.value) : null

	if (!box) {
		modelCenter.value = [0, 1, 0]
		modelSize.value = [1, 2, 1]
		return
	}

	const center = new THREE.Vector3()
	const size = new THREE.Vector3()

	box.getCenter(center)
	box.getSize(size)

	modelCenter.value = [center.x, center.y, center.z]
	modelSize.value = [Math.max(size.x, 0.001), Math.max(size.y, 0.001), Math.max(size.z, 0.001)]
}

const modelRotation = ref(props.initialRotation + Math.PI)
const isDragging = ref(false)
const previousX = ref(0)
const hasDragged = ref(false)

function onPointerDown(event: PointerEvent) {
	;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
	isDragging.value = true
	previousX.value = event.clientX
	hasDragged.value = false
}

function onPointerMove(event: PointerEvent) {
	if (!isDragging.value) return
	const deltaX = event.clientX - previousX.value
	modelRotation.value += deltaX * 0.01
	previousX.value = event.clientX
	hasDragged.value = true
}

function onPointerUp(event: PointerEvent) {
	isDragging.value = false
	;(event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId)
}

function onCanvasClick() {
	if (!hasDragged.value) {
		if (actions.value['interact']) {
			playRandomAnimation('interact')
		}
	}

	hasDragged.value = false
}

watch(selectedModelSrc, (src) => loadModel(src))
watch(
	() => props.lockFit,
	() => {
		fitLock.value = null
		lockFitState()
	},
)
watch(
	() => props.textureSrc,
	async (newSrc) => {
		isTextureLoaded.value = false
		texture.value = await loadAndApplyTexture(newSrc)
		if (scene.value && texture.value) {
			applyTexture(scene.value, texture.value)
		}
		isTextureLoaded.value = true
	},
)
watch(
	() => props.capeSrc,
	async (newCapeSrc) => {
		await loadAndApplyCapeTexture(newCapeSrc)
	},
)

watch(
	() => props.animationConfig,
	(newConfig) => {
		clearRandomAnimationTimer()

		if (mixer.value && newConfig.baseAnimation && actions.value[newConfig.baseAnimation]) {
			playAnimation(newConfig.baseAnimation)
			setupRandomAnimationLoop()
		}
	},
	{ deep: true },
)

onBeforeMount(async () => {
	try {
		isTextureLoaded.value = false
		texture.value = await loadAndApplyTexture(props.textureSrc)
		isTextureLoaded.value = true

		await loadModel(selectedModelSrc.value)

		if (props.capeSrc) {
			await loadAndApplyCapeTexture(props.capeSrc)
		}
	} catch (error) {
		console.error('Failed to initialize skin preview:', error)
	}
})

onUnmounted(() => {
	isUnmounted = true
	modelLoadVersion++
	resizeObserver?.disconnect()

	cleanupAnimationState(scene.value)
	disposeSceneMaterials(scene.value)
	scene.value = null
	transparentTexture.dispose()
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
