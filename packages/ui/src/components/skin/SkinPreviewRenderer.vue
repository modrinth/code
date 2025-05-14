<template>
  <div class="relative w-full h-full">
    <div class="absolute bottom-[18%] left-1/2 transform -translate-x-1/2 text-primary px-3 py-1 rounded text-md pointer-events-none z-10">
      Drag to Rotate
    </div>
    <div class="absolute top-[10%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md text-[225%] pointer-events-none z-10 font-minecraft text-secondary bg-bg-raised shadow-md border">
      {{ nametag }}
    </div>

    <TresCanvas
      shadows
      alpha
      :antialias="false"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointerleave="onPointerUp"
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
              :blending="AdditiveBlending"
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

      <TresAmbientLight :intensity="2" />
    </TresCanvas>
  </div>
</template>

<script setup lang="ts">
import * as THREE from 'three'
import { useGLTF } from '@tresjs/cientos'
import { useTexture, TresCanvas } from '@tresjs/core'
import { ref, computed } from 'vue'

const props = defineProps<{
  textureSrc: string
  modelSrc: string
  nametag?: string
}>()

const { scene } = await useGLTF(props.modelSrc)
await useTexture([props.textureSrc])

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
  isDragging.value = true
  previousX.value = event.clientX
}

const onPointerMove = (event: PointerEvent) => {
  if (!isDragging.value) return
  const deltaX = event.clientX - previousX.value
  modelRotation.value += deltaX * 0.01
  previousX.value = event.clientX
}

const onPointerUp = () => {
  isDragging.value = false
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

const AdditiveBlending = THREE.AdditiveBlending
</script>
