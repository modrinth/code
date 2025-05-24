<template>
  <div class="relative w-full h-full cursor-grab">
    <div
      class="absolute bottom-[18%] left-0 right-0 flex justify-center items-center mb-2 pointer-events-none z-10"
    >
      <span class="text-primary text-xs px-2 py-1 rounded-full backdrop-blur-sm">
        Drag to rotate
      </span>
    </div>
    <div
      v-if="nametag"
      class="absolute top-[13%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md text-[200%] pointer-events-none z-10 font-minecraft text-primary nametag-bg"
    >
      {{ nametag }}
    </div>

    <TresCanvas
      v-show="isReady"
      shadows
      alpha
      :antialias="antialias"
      :renderer-options="{
        outputColorSpace: THREE.SRGBColorSpace,
        toneMapping: THREE.NoToneMapping,
      }"
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
            :scale="[0.4 * scale, 0.4 * scale, 0.4 * scale]"
          >
            <TresCircleGeometry :args="[1, 32]" />
            <TresMeshBasicMaterial
              color="#000000"
              :opacity="0.3"
              transparent
              :depth-write="false"
            />
          </TresMesh>

          <TresMesh
            :position="[0, -0.1 * scale, 2]"
            :rotation="[-Math.PI / 2, 0, 0]"
            :scale="[0.75 * scale, 0.75 * scale, 0.75 * scale]"
          >
            <TresPlaneGeometry :args="[2, 2]" />
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
    </TresCanvas>

    <div v-if="!isReady" class="w-full h-full flex items-center justify-center">
      <div class="text-primary">Loading...</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as THREE from 'three'
import { useGLTF } from '@tresjs/cientos'
import { useTexture, TresCanvas } from '@tresjs/core'
import { shallowRef, ref, computed, watch, markRaw, onBeforeMount } from 'vue'
import {
  applyTexture,
  applyCapeTexture,
  attachCapeToBody,
  findBodyNode,
  createTransparentTexture,
  loadTexture as loadSkinTexture,
} from '@modrinth/utils'

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
  },
)

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

async function loadModel(src: string) {
  try {
    isModelLoaded.value = false
    const { scene: loadedScene } = await useGLTF(src)
    scene.value = markRaw(loadedScene)

    if (texture.value) {
      applyTexture(scene.value, texture.value)
      texture.value.needsUpdate = true
    }

    bodyNode.value = findBodyNode(loadedScene)
    capeAttached.value = false

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

const modelRotation = ref(props.initialRotation)
const isDragging = ref(false)
const previousX = ref(0)

const onPointerDown = (event: PointerEvent) => {
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
  isDragging.value = true
  previousX.value = event.clientX
}

const onPointerMove = (event: PointerEvent) => {
  if (!isDragging.value) return
  const deltaX = event.clientX - previousX.value
  modelRotation.value += deltaX * 0.01
  previousX.value = event.clientX
}

const onPointerUp = (event: PointerEvent) => {
  isDragging.value = false
  ;(event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId)
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
  grad.addColorStop(0, 'rgba(119,119,119,0.15)')
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
</script>

<style scoped lang="scss">
.nametag-bg {
  background: linear-gradient(308.68deg, rgba(0, 0, 0, 0) -52.46%, rgba(100, 100, 100, 0.1) 94.75%),
    rgba(0, 0, 0, 0.2);
  box-shadow:
    inset -0.5px -0.5px 0px rgba(0, 0, 0, 0.25),
    inset 0.5px 0.5px 0px rgba(255, 255, 255, 0.05);
}
</style>
