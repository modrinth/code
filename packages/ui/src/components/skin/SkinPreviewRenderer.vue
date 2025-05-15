<template>
  <div class="relative w-full h-full">
    <div class="absolute bottom-[18%] left-0 right-0 flex justify-center items-center mb-2 pointer-events-none z-10">
      <span class="text-primary text-xs px-2 py-1 rounded-full backdrop-blur-sm">
        Drag to rotate
      </span>
    </div>
    <div v-if="nametag" class="absolute top-[13%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md text-[200%] pointer-events-none z-10 font-minecraft text-white bg-black/60 shadow-md border-solid border-1 border-white/10">
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
          <!-- Apply the scale prop to the model group -->
          <Group :rotation="[0, modelRotation, 0]" :position="[0, -0.05 * scale, 1.95]" :scale="[0.8 * scale, 0.8 * scale, 0.8 * scale]">
            <primitive v-if="scene" :object="scene" />
          </Group>

          <!-- Scale the shadow accordingly -->
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

          <!-- Scale the radial gradient effect -->
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

      <!-- Use the fov prop for the camera -->
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
import {shallowRef, ref, computed, watch, markRaw} from 'vue'

const props = withDefaults(
  defineProps<{
    textureSrc: string
    slimModelSrc: string
    wideModelSrc: string
    variant?: 'SLIM' | 'CLASSIC' | 'UNKNOWN'
    nametag?: string
    antialias?: boolean
    scale?: number
    fov?: number
  }>(),
  {
    variant: 'CLASSIC',
    antialias: false,
    scale: 1,
    fov: 40
  }
)

const selectedModelSrc = computed(() =>
  props.variant === 'SLIM' ? props.slimModelSrc : props.wideModelSrc
)

const scene = shallowRef<THREE.Object3D | null>(null)

async function loadModel(src: string) {
  const { scene: loadedScene } = await useGLTF(src)
  scene.value = markRaw(loadedScene)
  applyTextureToScene(scene.value, texture.value)
  updateModelInfo()
}

watch(selectedModelSrc, src => loadModel(src), { immediate: true })

const texture = ref<THREE.Texture>(await useTexture([props.textureSrc]))

watch(
  () => props.textureSrc,
  async newSrc => {
    texture.value = await useTexture([newSrc])
    applyTextureToScene(scene.value, texture.value)
  }
)

function applyTextureToScene(root: THREE.Object3D | null, tex: THREE.Texture) {
  if (!root) return
  tex.colorSpace = THREE.SRGBColorSpace
  tex.flipY = false
  tex.magFilter = THREE.NearestFilter
  tex.minFilter = THREE.NearestFilter
  root.traverse(child => {
    if ((child as THREE.Mesh).isMesh) {
      const mesh = child as THREE.Mesh
      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
      materials.forEach((mat: THREE.MeshStandardMaterial) => {
        mat.map = tex
        mat.metalness = 0
        mat.color.set(0xffffff)
        mat.toneMapped = false
        mat.roughness = 1
        mat.needsUpdate = true
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

const modelRotation = ref(0)
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

applyTextureToScene(scene.value, texture.value)
loadModel(selectedModelSrc.value)
</script>
