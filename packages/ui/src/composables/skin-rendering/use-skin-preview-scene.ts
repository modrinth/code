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
import { applyEarsMod, removeEarsMod } from './use-ears-mod-features'

const SKIN_LAYER_DEPTH_BIAS = -1

function configureSkinPreviewMesh(mesh: THREE.Mesh) {
	const isSkinLayer = mesh.name.endsWith('_Layer')
	mesh.renderOrder = 0

	const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
	materials.forEach((material) => {
		if (!(material instanceof THREE.MeshStandardMaterial) || material.name === 'cape') return

		material.transparent = isSkinLayer
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
	earsTextureSrc,
	capeSrc,
	earsEnabled,
	initializeAnimations,
	cleanupAnimationState,
}: {
	selectedModelSrc: MaybeReadonlyRef<string>
	textureSrc: MaybeReadonlyRef<string>
	earsTextureSrc: MaybeReadonlyRef<string | undefined>
	capeSrc: MaybeReadonlyRef<string | undefined>
	earsEnabled: MaybeReadonlyRef<boolean>
	initializeAnimations: (loadedScene: THREE.Object3D, clips: THREE.AnimationClip[]) => void
	cleanupAnimationState: (root: THREE.Object3D | null) => void
}) {
	const scene = shallowRef<THREE.Object3D | null>(null)
	const lastCapeSrc = ref<string | undefined>(undefined)
	const loadedModelSrc = ref<string | undefined>(undefined)
	const loadedTextureSrc = ref<string | undefined>(undefined)
	const loadedEarsTextureSrc = ref<string | undefined>(undefined)
	const loadedCapeSrc = ref<string | undefined>(undefined)
	const texture = shallowRef<THREE.Texture | null>(null)
	const earsTexture = shallowRef<THREE.Texture | null>(null)
	const capeTexture = shallowRef<THREE.Texture | null>(null)
	const transparentTexture = createTransparentTexture()
	const modelCenter = ref<SkinPreviewTuple>([0, 1, 0])
	const modelSize = ref<SkinPreviewTuple>([1, 2, 1])
	const isModelLoaded = ref(false)
	const isTextureLoaded = ref(false)
	const hasEarsFeatures = ref(false)
	let modelLoadVersion = 0
	let textureLoadVersion = 0
	let earsTextureLoadVersion = 0
	let capeLoadVersion = 0
	let isUnmounted = false

	function applyTextureToLoadedModel() {
		if (
			!scene.value ||
			!texture.value ||
			loadedModelSrc.value !== selectedModelSrc.value ||
			loadedTextureSrc.value !== textureSrc.value
		) {
			return
		}

		applyTexture(scene.value, texture.value)
		const featureTextureSrc = earsTextureSrc.value
		const featureTexture = featureTextureSrc ? earsTexture.value : texture.value
		if (
			!featureTexture ||
			(featureTextureSrc && loadedEarsTextureSrc.value !== featureTextureSrc)
		) {
			removeEarsMod(scene.value)
			hasEarsFeatures.value = false
			return
		}

		hasEarsFeatures.value = applyEarsMod(scene.value, featureTexture, earsEnabled.value)
	}

	function applyCapeTextureToLoadedModel() {
		if (!scene.value || loadedModelSrc.value !== selectedModelSrc.value) return

		applyCapeTexture(
			scene.value,
			loadedCapeSrc.value === capeSrc.value ? capeTexture.value : null,
			transparentTexture,
		)
	}

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
			removeEarsMod(previousScene)
			disposeSceneMaterials(previousScene)
			scene.value = clonedScene
			loadedModelSrc.value = src

			applyTextureToLoadedModel()

			applyCapeTextureToLoadedModel()

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

		const loadVersion = ++capeLoadVersion
		lastCapeSrc.value = src

		let loadedCapeTexture: THREE.Texture | null = null
		if (src) {
			loadedCapeTexture = await loadAndApplyTexture(src)
		}
		if (isUnmounted || loadVersion !== capeLoadVersion) return

		capeTexture.value = loadedCapeTexture
		loadedCapeSrc.value = src
		applyCapeTextureToLoadedModel()
	}

	async function loadAndApplyEarsTexture(src: string | undefined) {
		const loadVersion = ++earsTextureLoadVersion
		hasEarsFeatures.value = false

		const loadedEarsTexture = src ? await loadAndApplyTexture(src) : null
		if (isUnmounted || loadVersion !== earsTextureLoadVersion) return

		earsTexture.value = loadedEarsTexture
		loadedEarsTextureSrc.value = src
		applyTextureToLoadedModel()
		updateModelInfo()
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
			const loadVersion = ++textureLoadVersion

			isTextureLoaded.value = false
			hasEarsFeatures.value = false
			const loadedTexture = await loadAndApplyTexture(newSrc)
			if (isUnmounted || loadVersion !== textureLoadVersion) return

			texture.value = loadedTexture
			loadedTextureSrc.value = newSrc
			applyTextureToLoadedModel()
			isTextureLoaded.value = true
		},
	)
	watch(
		() => earsTextureSrc.value,
		async (newEarsTextureSrc) => {
			await loadAndApplyEarsTexture(newEarsTextureSrc)
		},
	)
	watch(
		() => earsEnabled.value,
		() => {
			applyTextureToLoadedModel()
			updateModelInfo()
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
			loadedTextureSrc.value = textureSrc.value
			isTextureLoaded.value = true

			await loadAndApplyEarsTexture(earsTextureSrc.value)
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
		textureLoadVersion++
		earsTextureLoadVersion++
		capeLoadVersion++

		cleanupAnimationState(scene.value)
		removeEarsMod(scene.value)
		disposeSceneMaterials(scene.value)
		scene.value = null
		transparentTexture.dispose()
	})

	return {
		hasEarsFeatures,
		isModelLoaded,
		isTextureLoaded,
		modelCenter,
		modelSize,
		scene,
	}
}
