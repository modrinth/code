<template>
  <div class="relative w-full h-full">
    <div class="absolute bottom-[18%] left-0 right-0 flex justify-center items-center mb-2 pointer-events-none z-10">
      <span class="text-primary text-xs px-2 py-1 rounded-full backdrop-blur-sm">
        Drag to rotate
      </span>
    </div>
    <div v-if="nametag" class="absolute top-[13%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md text-[200%] pointer-events-none z-10 font-minecraft text-primary nametag-bg">
      {{ nametag }}
    </div>

    <TresCanvas
      shadows
      alpha
      :antialias="antialias"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointerleave="onPointerUp"
      :rendererOptions="{
        outputColorSpace: THREE.SRGBColorSpace,
        toneMapping: THREE.NoToneMapping
      }"
    >
      <Suspense>
        <Group>
          <Group :rotation="[0, modelRotation, 0]" :position="[0, -0.05 * scale, 1.95]" :scale="[0.8 * scale, 0.8 * scale, 0.8 * scale]">
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
              :opacity="0.6"
              transparent
              :depth-write="false"
            />
          </TresMesh>

          <TresMesh :position="[0, -0.1 * scale, 2]" :rotation="[-Math.PI / 2, 0, 0]" :scale="[0.75 * scale, 0.75 * scale, 0.75 * scale]">
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
        :makeDefault="true"
        :fov="fov"
        :position="[0, 1.5, -3.25]"
        :look-at="target"
      />

      <TresAmbientLight :intensity="2" />
    </TresCanvas>
  </div>
</template>

<script setup lang="ts">
import * as THREE from 'three'
import { useGLTF } from '@tresjs/cientos'
import { useTexture, TresCanvas } from '@tresjs/core'
import { shallowRef, ref, computed, watch, markRaw, onBeforeMount } from 'vue'

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
    fov?: number,
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
  }
)

const selectedModelSrc = computed(() =>
  props.variant === 'SLIM' ? props.slimModelSrc : props.wideModelSrc
)

const scene = shallowRef<THREE.Object3D | null>(null)
const capeScene = shallowRef<THREE.Object3D | null>(null)
const bodyNode = shallowRef<THREE.Object3D | null>(null)
const capeAttached = ref(false)
const lastCapeSrc = ref<string | undefined>(undefined)
const texture = shallowRef<THREE.Texture | null>(null)
const capeTexture = shallowRef<THREE.Texture | null>(null)
const transparentTexture = createTransparentTexture()

function createTransparentTexture(): THREE.Texture {
  const canvas = document.createElement('canvas')
  canvas.width = canvas.height = 1
  const ctx = canvas.getContext('2d') as CanvasRenderingContext2D
  ctx.clearRect(0, 0, 1, 1)

  const texture = new THREE.CanvasTexture(canvas)
  texture.needsUpdate = true
  texture.colorSpace = THREE.SRGBColorSpace
  texture.flipY = false
  texture.magFilter = THREE.NearestFilter
  texture.minFilter = THREE.NearestFilter

  return texture
}

async function loadModel(src: string) {
  const { scene: loadedScene } = await useGLTF(src)
  scene.value = markRaw(loadedScene)

  if (texture.value) {
    applyTextureToScene(scene.value, texture.value)
  }

  bodyNode.value = null
  loadedScene.traverse(node => {
    if (node.name === 'Body') {
      bodyNode.value = node
    }
  })

  capeAttached.value = false

  if (capeScene.value && bodyNode.value) {
    attachCapeToBody()
  }

  updateModelInfo()
}

async function loadCape(src: string) {
  if (!src) {
    capeScene.value = null
    return
  }

  const { scene: loadedCape } = await useGLTF(src)
  capeScene.value = markRaw(loadedCape)

  applyCapeTexture(capeScene.value, capeTexture.value)

  if (bodyNode.value && !capeAttached.value) {
    attachCapeToBody()
  }
}

async function loadAndApplyTexture(src: string) {
  if (!src) return null
  const tex = await useTexture([src])
  tex.colorSpace = THREE.SRGBColorSpace
  tex.flipY = false
  tex.magFilter = THREE.NearestFilter
  tex.minFilter = THREE.NearestFilter
  return tex
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
    applyCapeTexture(capeScene.value, capeTexture.value)
  }

  if (capeScene.value && bodyNode.value) {
    if (!src && capeAttached.value && capeScene.value.parent) {
      capeScene.value.parent.remove(capeScene.value)
      capeAttached.value = false
    } else if (src && !capeAttached.value) {
      attachCapeToBody()
    }
  }
}

function attachCapeToBody() {
  if (!bodyNode.value || !capeScene.value || capeAttached.value) return

  if (capeScene.value.parent) {
    capeScene.value.parent.remove(capeScene.value)
  }

  capeScene.value.position.set(0, -1, -0.01)
  capeScene.value.rotation.set(0, -Math.PI / 2, 0)

  bodyNode.value.add(capeScene.value)
  capeAttached.value = true
}

function applyTextureToScene(root: THREE.Object3D | null, tex: THREE.Texture | null) {
  if (!root || !tex) return

  root.traverse(child => {
    if ((child as THREE.Mesh).isMesh) {
      const mesh = child as THREE.Mesh

      if (mesh.name === "Cape") return

      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

      materials.forEach((mat, _index, _array) => {
        const standardMat = mat as THREE.MeshStandardMaterial
        standardMat.map = tex
        standardMat.metalness = 0
        standardMat.color.set(0xffffff)
        standardMat.toneMapped = false
        standardMat.roughness = 1
        standardMat.needsUpdate = true
      })
    }
  })
}

function applyCapeTexture(root: THREE.Object3D | null, tex: THREE.Texture | null) {
  if (!root) return

  root.traverse(child => {
    if ((child as THREE.Mesh).isMesh) {
      const mesh = child as THREE.Mesh

      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

      materials.forEach((mat, _index, _array) => {
        const standardMat = mat as THREE.MeshStandardMaterial
        standardMat.map = tex || transparentTexture
        standardMat.transparent = true
        standardMat.metalness = 0
        standardMat.color.set(0xffffff)
        standardMat.toneMapped = false
        standardMat.roughness = 1
        standardMat.side = THREE.DoubleSide
        standardMat.needsUpdate = true
      })
    }
  })
}

const centre = ref<[number, number, number]>([0, 1, 0])
const modelHeight = ref(1.4)

function updateModelInfo() {
  if (!scene.value) return
  const bbox = new THREE.Box3().setFromObject(scene.value)
  const mid = new THREE.Vector3()
  bbox.getCenter(mid)
  centre.value = [mid.x, mid.y, mid.z]
  modelHeight.value = bbox.max.y - bbox.min.y
}

const target = computed(() => centre.value)

const modelRotation = ref(props.initialRotation)
const isDragging = ref(false)
const previousX = ref(0)

const onPointerDown = (event: PointerEvent) => {
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
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
  isDragging.value = false;
  (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId)
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
  grad.addColorStop(0, 'rgba(119,119,119,0.25)')
  grad.addColorStop(1, 'rgba(255,255,255,0)')
  ctx.fillStyle = grad
  ctx.fillRect(0, 0, size, size)
  return new THREE.CanvasTexture(canvas)
}

watch(selectedModelSrc, src => loadModel(src))
watch(() => props.capeModelSrc, src => src && loadCape(src))
watch(() => props.textureSrc, async newSrc => {
  texture.value = await loadAndApplyTexture(newSrc)
  if (scene.value && texture.value) {
    applyTextureToScene(scene.value, texture.value)
  }
})
watch(() => props.capeSrc, async newCapeSrc => {
  await loadAndApplyCapeTexture(newCapeSrc)
})

onBeforeMount(async () => {
  texture.value = await loadAndApplyTexture(props.textureSrc)

  await loadModel(selectedModelSrc.value)

  if (props.capeSrc) {
    await loadAndApplyCapeTexture(props.capeSrc)
  }

  if (props.capeModelSrc) {
    await loadCape(props.capeModelSrc)
  }
})
</script>

<style scoped lang="scss">
.nametag-bg {
  background: linear-gradient(308.68deg, rgba(0, 0, 0, 0) -52.46%, rgba(100, 100, 100, 0.1) 94.75%), rgba(0, 0, 0, 0.2);
  box-shadow: inset -0.5px -0.5px 0px rgba(0, 0, 0, 0.25), inset 0.5px 0.5px 0px rgba(255, 255, 255, 0.05);
}
</style>
