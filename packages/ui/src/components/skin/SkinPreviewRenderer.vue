<template>
  <div class="relative w-full h-full">
    <div class="absolute bottom-[18%] left-1/2 transform -translate-x-1/2 text-primary px-3 py-1 rounded text-md pointer-events-none z-10">
      Drag to Rotate
    </div>
    <div
      class="absolute top-[5%] left-1/2 transform -translate-x-1/2 px-3 py-1 rounded-md text-[190%] pointer-events-none z-10 font-minecraft
      text-secondary bg-bg-raised shadow-md border"
    >
      {{ nametag }}
    </div>

    <TresCanvas shadows alpha :antialias="false" @pointerdown="onPointerDown" @pointermove="onPointerMove" @pointerup="onPointerUp" @pointerleave="onPointerUp">
      <Suspense>
        <Group>
          <Group
            ref="modelGroup"
            :rotation="[0, modelRotation, 0]"
            :position="[0, 0, 2]"
          >
            <primitive
              v-if="scene"
              ref="modelRef"
              :object="scene"
            />
          </Group>

          <TresMesh :position="[0, -0.1, 2]" :rotation="[-Math.PI / 2, 0, 0]" :scale="[0.75, 0.75, 0.75]">
            <TresPlaneGeometry :args="[2, 2]" />
            <TresShaderMaterial
              transparent
              :depth-write="false"
              :uniforms="spotlightUniforms"
              :vertex-shader="spotlightVertexShader"
              :fragment-shader="spotlightFragmentShader"
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
import {Html, useGLTF} from '@tresjs/cientos'
import { useTexture, TresCanvas } from '@tresjs/core'
import { ref, computed, onMounted } from 'vue'

const props = defineProps<{
  textureSrc: string
  modelSrc: string
  nametag?: string
}>()

const { scene } = await useGLTF(props.modelSrc)
await useTexture([props.textureSrc])

const modelRef = ref(null)
const modelGroup = ref(null)
const cameraRef = ref(null)
const nametagRef = ref(null)
const centre = ref<[number, number, number]>([0, 1, 0])
const modelHeight = ref(1.8)

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

const spotlightUniforms = {
  uColor: { value: new THREE.Color(1.0, 1.0, 1.0) },
  uIntensity: { value: 3 },
  uRadius: { value: 1 }
}

// language=GLSL
const spotlightVertexShader = `
  attribute vec3 position;
  attribute vec2 uv;
  varying vec2 vUv;

  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

// language=GLSL
const spotlightFragmentShader = `
  precision highp float;

  uniform vec3 uColor;
  uniform float uIntensity;
  uniform float uRadius;

  varying vec2 vUv;

  void main() {
    float dist = distance(vUv, vec2(0.5));
    float alpha = 1.0 - smoothstep(0.0, uRadius, dist);

    vec3 col = uColor * uIntensity * alpha;
    gl_FragColor = vec4(col, alpha);
  }
`;
</script>
