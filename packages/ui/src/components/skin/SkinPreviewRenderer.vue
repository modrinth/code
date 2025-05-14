<template>
  <div class="relative w-full h-full">
    <div class="absolute bottom-[18%] left-1/2 transform -translate-x-1/2 text-primary px-3 py-1 rounded text-md pointer-events-none z-10">
      Drag to rotate
    </div>
    <div class="absolute top-[10%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md text-[225%] pointer-events-none z-10 font-minecraft text-secondary bg-bg-raised shadow-md border">
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
          <Group ref="modelGroup" :rotation="[0, modelRotation, 0]" :position="[0, -0.05, 1.95]" :scale="[0.8, 0.8, 0.8]">
            <primitive v-if="scene" ref="modelRef" :object="scene" />
          </Group>

          <TresMesh
            :position="[0, -0.095, 2]"
            :rotation="[-Math.PI / 2, 0, 0]"
            :scale="[0.4, 0.4, 0.4]"
          >
            <TresCircleGeometry :args="[1, 32]" />
            <TresMeshBasicMaterial
              color="#000000"
              :opacity="0.6"
              transparent
              :depth-write="false"
            />
          </TresMesh>

          <TresMesh :position="[0, -0.1, 2]" :rotation="[-Math.PI / 2, 0, 0]" :scale="[0.75, 0.75, 0.75]">
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
        ref="cameraRef"
        :makeDefault="true"
        :fov="40"
        :position="[0, 1.5, -3.25]"
        :look-at="target"
      />

      <TresAmbientLight
        :intensity="2"
      />
    </TresCanvas>
  </div>
</template>

<script setup lang="ts">
import * as THREE from 'three'
import { useGLTF } from '@tresjs/cientos'
import { useTexture, TresCanvas } from '@tresjs/core'
import {ref, computed, watch} from 'vue'

const props = withDefaults(defineProps<{
  textureSrc: string
  modelSrc: string
  nametag?: string
  antialias?: boolean
}>(), {
  antialias: false,
})

const { scene } = await useGLTF(props.modelSrc)

let texture = await useTexture([props.textureSrc])
applyTextureToScene(scene, texture);

watch(
  () => props.textureSrc,
  async newSrc => {
    texture = await useTexture([newSrc])
    applyTextureToScene(scene, texture)
  }
)

function applyTextureToScene(root: THREE.Object3D | null, tex: THREE.Texture) {
  texture.colorSpace = THREE.SRGBColorSpace
  texture.flipY = false
  texture.magFilter = THREE.NearestFilter
  texture.minFilter = THREE.NearestFilter

  if (!root) return
  root.traverse(child => {
    if ((child as THREE.Mesh).isMesh) {
      const mesh = child as THREE.Mesh
      const setProps = (mat: THREE.Material) => {
        const m = mat as THREE.MeshStandardMaterial
        m.map = tex
        m.metalness = 0
        m.color.set(0xffffff)
        m.toneMapped = false
        m.roughness = 1
        m.needsUpdate = true
      }

      if (Array.isArray(mesh.material)) mesh.material.forEach(setProps)
      else setProps(mesh.material)
    }
  })
}

const modelRef = ref<THREE.Object3D | null>(null)
const modelGroup = ref<THREE.Group | null>(null)
const cameraRef = ref<THREE.PerspectiveCamera | null>(null)
const centre = ref<[number, number, number]>([0, 1, 0])
const modelHeight = ref(1.4)

if (scene) {
  const bbox = new THREE.Box3().setFromObject(scene)
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
</script>
