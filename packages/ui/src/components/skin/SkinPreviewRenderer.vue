<template>
  <div ref="skinPreviewContainer" class="relative w-full h-full cursor-grab" @click="onCanvasClick">
    <div
      class="absolute bottom-[18%] left-0 right-0 flex flex-col justify-center items-center mb-2 pointer-events-none z-10 gap-2"
    >
      <span class="text-primary text-xs px-2 py-1 rounded-full backdrop-blur-sm">
        Drag to rotate
      </span>
    </div>
    <div
      class="absolute bottom-[10%] left-0 right-0 flex justify-center items-center pointer-events-auto z-10"
    >
      <slot name="subtitle" />
    </div>
    <div
      v-if="nametag"
      class="absolute top-[18%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md pointer-events-none z-10 font-minecraft text-gray nametag-bg transition-all duration-200"
      :style="{ fontSize: nametagFontSize }"
    >
      {{ nametagText }}
    </div>

    <TresCanvas
      shadows
      alpha
      :antialias="antialias"
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
        <Group>
          <Group
            :rotation="[0, modelRotation, 0]"
            :position="[0, -0.05 * scale, 1.95]"
            :scale="[0.8 * scale, 0.8 * scale, 0.8 * scale]"
          >
            <primitive v-if="scene" :object="scene" />
          </Group>

          <TresMesh
            :position="[0, -0.095 * scale, 2]"
            :rotation="[-Math.PI / 2, 0, 0]"
            :scale="[0.5 * 0.75 * scale, 0.5 * 0.75 * scale, 0.5 * 0.75 * scale]"
          >
            <TresCircleGeometry :args="[1, 128]" />
            <TresMeshBasicMaterial
              color="#000000"
              :opacity="0.2"
              transparent
              :depth-write="false"
            />
          </TresMesh>

          <TresMesh
            :position="[0, -0.1 * scale, 2]"
            :rotation="[-Math.PI / 2, 0, 0]"
            :scale="[0.75 * scale, 0.75 * scale, 0.75 * scale]"
          >
            <TresCircleGeometry :args="[1, 128]" />
            <TresMeshBasicMaterial
              :map="radialTexture"
              transparent
              :depth-write="false"
              :blending="THREE.AdditiveBlending"
            />
          </TresMesh>
        </Group>
      </Suspense>

      <TresPerspectiveCamera
        :make-default.camel="true"
        :fov="fov"
        :position="[0, 1.5, -3.25]"
        :look-at="target"
      />

      <TresAmbientLight :intensity="2" />
      <TresDirectionalLight :position="[2, 4, 3]" :intensity="1.2" :cast-shadow="true" />
    </TresCanvas>

    <div
      v-if="!isReady"
      class="w-full h-full flex items-center justify-center transition-opacity duration-500"
      :class="{ 'opacity-100': !isReady, 'opacity-0': isReady }"
    >
      <div class="text-primary">Loading...</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as THREE from 'three'
import { useGLTF } from '@tresjs/cientos'
import { useTexture, TresCanvas, useRenderLoop } from '@tresjs/core'
import {
  shallowRef,
  ref,
  computed,
  watch,
  markRaw,
  onBeforeMount,
  onUnmounted,
  toRefs,
  useTemplateRef,
} from 'vue'
import {
  applyTexture,
  applyCapeTexture,
  attachCapeToBody,
  findBodyNode,
  createTransparentTexture,
  loadTexture as loadSkinTexture,
} from '@modrinth/utils'
import { useDynamicFontSize } from '../../composables'

interface AnimationConfig {
  baseAnimation: string
  randomAnimations: string[]
  randomAnimationInterval?: number
  transitionDuration?: number
}

const props = withDefaults(
  defineProps<{
    textureSrc: string
    slimModelSrc: string
    wideModelSrc: string
    capeModelSrc?: string
    capeSrc?: string
    variant?: 'SLIM' | 'CLASSIC' | 'UNKNOWN'
    nametag?: string
    antialias?: boolean
    scale?: number
    fov?: number
    initialRotation?: number
    animationConfig?: AnimationConfig
  }>(),
  {
    variant: 'CLASSIC',
    antialias: false,
    scale: 1,
    fov: 40,
    capeModelSrc: '',
    capeSrc: undefined,
    initialRotation: 15.75,
    nametag: undefined,
    animationConfig: () => ({
      baseAnimation: 'idle',
      randomAnimations: ['idle_sub_1', 'idle_sub_2', 'idle_sub_3'],
      randomAnimationInterval: 8000,
      transitionDuration: 0.2,
    }),
  },
)

const skinPreviewContainer = useTemplateRef<HTMLElement>('skinPreviewContainer')
const nametagText = computed(() => props.nametag)

const { fontSize: nametagFontSize } = useDynamicFontSize({
  containerElement: skinPreviewContainer,
  text: nametagText,
  baseFontSize: 1.8,
  minFontSize: 1.25,
  maxFontSize: 2,
  padding: 24,
  fontFamily: 'inherit',
})

const selectedModelSrc = computed(() =>
  props.variant === 'SLIM' ? props.slimModelSrc : props.wideModelSrc,
)

const scene = shallowRef<THREE.Object3D | null>(null)
const capeScene = shallowRef<THREE.Object3D | null>(null)
const bodyNode = shallowRef<THREE.Object3D | null>(null)
const capeAttached = ref(false)
const lastCapeSrc = ref<string | undefined>(undefined)
const texture = shallowRef<THREE.Texture | null>(null)
const capeTexture = shallowRef<THREE.Texture | null>(null)
const transparentTexture = createTransparentTexture()

const isModelLoaded = ref(false)
const isTextureLoaded = ref(false)
const isReady = computed(() => isModelLoaded.value && isTextureLoaded.value)

const mixer = ref<THREE.AnimationMixer | null>(null)
const actions = ref<Record<string, THREE.AnimationAction>>({})
const clock = new THREE.Clock()
const currentAnimation = ref<string>('')
const randomAnimationTimer = ref<number | null>(null)
const lastRandomAnimation = ref<string>('')

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

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const onFinished = (event: any) => {
      if (event.action === action) {
        mixer.value?.removeEventListener('finished', onFinished)
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

    mixer.value.addEventListener('finished', onFinished)
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

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const onFinished = (event: any) => {
    if (event.action === action) {
      mixer.value?.removeEventListener('finished', onFinished)
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

  mixer.value.addEventListener('finished', onFinished)
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

async function loadModel(src: string) {
  try {
    isModelLoaded.value = false
    const { scene: loadedScene, animations } = await useGLTF(src)
    scene.value = markRaw(loadedScene)

    if (texture.value) {
      applyTexture(scene.value, texture.value)
      texture.value.needsUpdate = true
    }

    bodyNode.value = findBodyNode(loadedScene)
    capeAttached.value = false

    if (animations && animations.length > 0) {
      initializeAnimations(loadedScene, animations)
    }

    updateModelInfo()
    isModelLoaded.value = true
  } catch (error) {
    console.error('Failed to load model:', error)
    isModelLoaded.value = false
  }
}

async function loadCape(src: string) {
  if (!src) {
    capeScene.value = null
    return
  }

  try {
    const { scene: loadedCape } = await useGLTF(src)
    capeScene.value = markRaw(loadedCape)

    applyCapeTexture(capeScene.value, capeTexture.value, transparentTexture)

    if (bodyNode.value && !capeAttached.value) {
      attachCapeToBodyWrapper()
    }
  } catch (error) {
    console.error('Failed to load cape:', error)
    capeScene.value = null
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

  if (capeScene.value) {
    applyCapeTexture(capeScene.value, capeTexture.value, transparentTexture)
  }

  if (capeScene.value && bodyNode.value) {
    if (!src && capeAttached.value && capeScene.value.parent) {
      capeScene.value.parent.remove(capeScene.value)
      capeAttached.value = false
    } else if (src && !capeAttached.value) {
      attachCapeToBodyWrapper()
    }
  }
}

function attachCapeToBodyWrapper() {
  if (!bodyNode.value || !capeScene.value || capeAttached.value) return

  attachCapeToBody(bodyNode.value, capeScene.value)
  capeAttached.value = true
}

const centre = ref<[number, number, number]>([0, 1, 0])
const modelHeight = ref(1.4)

function updateModelInfo() {
  if (!scene.value) return
  try {
    const bbox = new THREE.Box3().setFromObject(scene.value)
    const mid = new THREE.Vector3()
    bbox.getCenter(mid)
    centre.value = [mid.x, mid.y, mid.z]
    modelHeight.value = bbox.max.y - bbox.min.y
  } catch (error) {
    console.error('Failed to update model info:', error)
  }
}

const target = computed(() => centre.value)

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

const radialTexture = createRadialTexture(512)
radialTexture.minFilter = THREE.LinearFilter
radialTexture.magFilter = THREE.LinearFilter
radialTexture.wrapS = radialTexture.wrapT = THREE.ClampToEdgeWrapping

function createRadialTexture(size: number): THREE.CanvasTexture {
  const canvas = document.createElement('canvas')
  canvas.width = canvas.height = size
  const ctx = canvas.getContext('2d') as CanvasRenderingContext2D
  const grad = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2)
  grad.addColorStop(0, 'rgba(119,119,119,0.1)')
  grad.addColorStop(0.9, 'rgba(255,255,255,0)')
  ctx.fillStyle = grad
  ctx.fillRect(0, 0, size, size)
  return new THREE.CanvasTexture(canvas)
}

watch(
  [bodyNode, capeScene, isModelLoaded],
  ([newBodyNode, newCapeScene, modelLoaded]) => {
    if (newBodyNode && newCapeScene && modelLoaded && !capeAttached.value) {
      attachCapeToBodyWrapper()
    }
  },
  { immediate: true },
)

watch(capeScene, (newCapeScene) => {
  if (newCapeScene && bodyNode.value && isModelLoaded.value && !capeAttached.value) {
    attachCapeToBodyWrapper()
  }
})

watch(selectedModelSrc, (src) => loadModel(src))
watch(
  () => props.capeModelSrc,
  (src) => src && loadCape(src),
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
    if (randomAnimationTimer.value) {
      clearTimeout(randomAnimationTimer.value)
      randomAnimationTimer.value = null
    }

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

    if (props.capeModelSrc) {
      await loadCape(props.capeModelSrc)
    }
  } catch (error) {
    console.error('Failed to initialize skin preview:', error)
  }
})

onUnmounted(() => {
  if (randomAnimationTimer.value) {
    clearTimeout(randomAnimationTimer.value)
  }

  if (mixer.value) {
    mixer.value.stopAllAction()
    mixer.value = null
  }
})
</script>

<style scoped lang="scss">
.nametag-bg {
  background: linear-gradient(
      308.68deg,
      rgba(50, 50, 50, 0.2) -52.46%,
      rgba(100, 100, 100, 0.2) 94.75%
    ),
    rgba(0, 0, 0, 0.2);
  box-shadow:
    inset -0.5px -0.5px 0px rgba(0, 0, 0, 0.25),
    inset 0.5px 0.5px 0px rgba(255, 255, 255, 0.05);
}
</style>
