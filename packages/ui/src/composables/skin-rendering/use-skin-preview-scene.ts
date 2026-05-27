import { useGLTF } from '@tresjs/cientos'
import { useTexture } from '@tresjs/core'
import * as THREE from 'three'
import { clone as cloneSkeleton } from 'three/examples/jsm/utils/SkeletonUtils.js'
import {
	type ComputedRef,
	markRaw,
	onBeforeMount,
	onUnmounted,
	type Ref,
	ref,
	shallowRef,
	watch,
} from 'vue'

import {
	applyCapeTexture,
	applyTexture,
	createTransparentTexture,
	loadTexture as loadSkinTexture,
} from '#ui/utils/webgl/skin-rendering.ts'

import type { SkinPreviewTuple } from './types'

const SKIN_LAYER_DEPTH_BIAS = -1

function configureSkinPreviewMesh(mesh: THREE.Mesh) {
	const isSkinLayer = mesh.name.endsWith('_Layer')
	mesh.renderOrder = 0

	const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
	materials.forEach((material) => {
		if (!(material instanceof THREE.MeshStandardMaterial) || material.name === 'cape') return

		material.transparent = false
		material.alphaTest = 0.1
		material.depthTest = true
		material.depthWrite = true
		material.polygonOffset = isSkinLayer
		material.polygonOffsetFactor = isSkinLayer ? SKIN_LAYER_DEPTH_BIAS : 0
		material.polygonOffsetUnits = isSkinLayer ? SKIN_LAYER_DEPTH_BIAS : 0
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

type MaybeReadonlyRef<T> = Ref<T> | ComputedRef<T>

export function useSkinPreviewScene({
	selectedModelSrc,
	textureSrc,
	capeSrc,
	initializeAnimations,
	cleanupAnimationState,
}: {
	selectedModelSrc: MaybeReadonlyRef<string>
	textureSrc: MaybeReadonlyRef<string>
	capeSrc: MaybeReadonlyRef<string | undefined>
	initializeAnimations: (loadedScene: THREE.Object3D, clips: THREE.AnimationClip[]) => void
	cleanupAnimationState: (root: THREE.Object3D | null) => void
}) {
	const scene = shallowRef<THREE.Object3D | null>(null)
	const lastCapeSrc = ref<string | undefined>(undefined)
	const texture = shallowRef<THREE.Texture | null>(null)
	const capeTexture = shallowRef<THREE.Texture | null>(null)
	const transparentTexture = createTransparentTexture()
	const modelCenter = ref<SkinPreviewTuple>([0, 1, 0])
	const modelSize = ref<SkinPreviewTuple>([1, 2, 1])
	const isModelLoaded = ref(false)
	const isTextureLoaded = ref(false)
	let modelLoadVersion = 0
	let isUnmounted = false

	async function loadModel(src: string) {
		const loadVersion = ++modelLoadVersion

		try {
			isModelLoaded.value = false
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

	watch(
		() => selectedModelSrc.value,
		(src) => loadModel(src),
	)
	watch(
		() => textureSrc.value,
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
		() => capeSrc.value,
		async (newCapeSrc) => {
			await loadAndApplyCapeTexture(newCapeSrc)
		},
	)

	onBeforeMount(async () => {
		try {
			isTextureLoaded.value = false
			texture.value = await loadAndApplyTexture(textureSrc.value)
			isTextureLoaded.value = true

			await loadModel(selectedModelSrc.value)

			if (capeSrc.value) {
				await loadAndApplyCapeTexture(capeSrc.value)
			}
		} catch (error) {
			console.error('Failed to initialize skin preview:', error)
		}
	})

	onUnmounted(() => {
		isUnmounted = true
		modelLoadVersion++

		cleanupAnimationState(scene.value)
		disposeSceneMaterials(scene.value)
		scene.value = null
		transparentTexture.dispose()
	})

	return {
		isModelLoaded,
		isTextureLoaded,
		modelCenter,
		modelSize,
		scene,
	}
}
