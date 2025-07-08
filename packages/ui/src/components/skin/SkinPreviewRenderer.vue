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
      :antialias="true"
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

          <!-- <TresMesh
            :position="[0, -0.095 * scale, 2]"
            :rotation="[-Math.PI / 2, 0, 0]"
            :scale="[0.4 * 0.75 * scale, 0.4 * 0.75 * scale, 0.4 * 0.75 * scale]"
          >
            <TresCircleGeometry :args="[1, 128]" />
            <TresMeshBasicMaterial
              color="#000000"
              :opacity="0.5"
              transparent
              :depth-write="false"
            />
          </TresMesh> -->
        </Group>
      </Suspense>

      <Suspense>
        <EffectComposerPmndrs>
          <FXAAPmndrs />
        </EffectComposerPmndrs>
      </Suspense>

      <Suspense>
        <TresMesh
          :position="[0, -0.1 * scale, 2]"
          :rotation="[-Math.PI / 2, 0, 0]"
          :scale="[0.75 * scale, 0.75 * scale, 0.75 * scale]"
        >
          <TresCircleGeometry :args="[1, 128]" />
          <TresShaderMaterial v-bind="radialSpotlightShader" />
        </TresMesh>
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
import { EffectComposerPmndrs, FXAAPmndrs } from '@tresjs/post-processing'
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
  createTransparentTexture,
  loadTexture as loadSkinTexture,
} from '@modrinth/utils'
import { useDynamicFontSize } from '../../composables'
import { ClassicPlayerModel, SlimPlayerModel } from '@modrinth/assets'

interface AnimationConfig {
  baseAnimation: string
  randomAnimations: string[]
  randomAnimationInterval?: number
  transitionDuration?: number
}

const props = withDefaults(
  defineProps<{
    textureSrc: string
    capeSrc?: string
    variant?: 'SLIM' | 'CLASSIC' | 'UNKNOWN'
    nametag?: string
    scale?: number
    fov?: number
    initialRotation?: number
    animationConfig?: AnimationConfig
  }>(),
  {
    variant: 'CLASSIC',
    scale: 1,
    fov: 40,
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
  props.variant === 'SLIM' ? SlimPlayerModel : ClassicPlayerModel,
)

const scene = shallowRef<THREE.Object3D | null>(null)
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
    }

    applyCapeTexture(scene.value, capeTexture.value, transparentTexture)

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

watch(selectedModelSrc, (src) => loadModel(src))
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
