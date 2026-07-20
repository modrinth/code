import * as THREE from 'three'
import type { GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'

export interface SkinRendererConfig {
	textureColorSpace?: THREE.ColorSpace
	textureFlipY?: boolean
	textureMagFilter?: THREE.MagnificationTextureFilter
	textureMinFilter?: THREE.MinificationTextureFilter
}

const modelCache: Map<string, GLTF> = new Map()
const modelPromiseCache: Map<string, Promise<GLTF>> = new Map()
const textureCache: Map<string, THREE.Texture> = new Map()
const texturePromiseCache: Map<string, Promise<THREE.Texture>> = new Map()
const SKIN_TEXTURE_SIZE = 64
const SKIN_UV_INSET_VERSION = 1

export async function loadModel(modelUrl: string): Promise<GLTF> {
	if (modelCache.has(modelUrl)) {
		return modelCache.get(modelUrl)!
	}

	if (modelPromiseCache.has(modelUrl)) {
		return modelPromiseCache.get(modelUrl)!
	}

	const loader = new GLTFLoader()
	const promise = new Promise<GLTF>((resolve, reject) => {
		loader.load(
			modelUrl,
			(gltf) => {
				modelCache.set(modelUrl, gltf)
				resolve(gltf)
			},
			undefined,
			reject,
		)
	}).finally(() => {
		modelPromiseCache.delete(modelUrl)
	})

	modelPromiseCache.set(modelUrl, promise)
	return promise
}

export async function loadTexture(
	textureUrl: string,
	config: SkinRendererConfig = {},
): Promise<THREE.Texture> {
	const cacheKey = `${textureUrl}_${JSON.stringify(config)}`

	if (textureCache.has(cacheKey)) {
		return textureCache.get(cacheKey)!
	}

	if (texturePromiseCache.has(cacheKey)) {
		return texturePromiseCache.get(cacheKey)!
	}

	const textureLoader = new THREE.TextureLoader()
	const promise = new Promise<THREE.Texture>((resolve, reject) => {
		textureLoader.load(
			textureUrl,
			(texture) => {
				texture.colorSpace = config.textureColorSpace ?? THREE.SRGBColorSpace
				texture.flipY = config.textureFlipY ?? false
				texture.magFilter = config.textureMagFilter ?? THREE.NearestFilter
				texture.minFilter = config.textureMinFilter ?? THREE.NearestFilter

				textureCache.set(cacheKey, texture)
				resolve(texture)
			},
			undefined,
			reject,
		)
	}).finally(() => {
		texturePromiseCache.delete(cacheKey)
	})

	texturePromiseCache.set(cacheKey, promise)
	return promise
}

function applyMap(mat: THREE.MeshStandardMaterial, texture: THREE.Texture | null): boolean {
	const hadMap = mat.map !== null
	const hasMap = texture !== null

	if (mat.map !== texture) {
		mat.map = texture
	}

	return hadMap !== hasMap
}

function setShaderMaterialProperties(
	mat: THREE.MeshStandardMaterial,
	properties: {
		alphaTest: number
		flatShading: boolean
		side: THREE.Side
		toneMapped: boolean
		transparent?: boolean
	},
): boolean {
	let needsUpdate = false

	if (mat.alphaTest !== properties.alphaTest) {
		mat.alphaTest = properties.alphaTest
		needsUpdate = true
	}

	if (mat.flatShading !== properties.flatShading) {
		mat.flatShading = properties.flatShading
		needsUpdate = true
	}

	if (mat.side !== properties.side) {
		mat.side = properties.side
		needsUpdate = true
	}

	if (mat.toneMapped !== properties.toneMapped) {
		mat.toneMapped = properties.toneMapped
		needsUpdate = true
	}

	if (properties.transparent !== undefined && mat.transparent !== properties.transparent) {
		mat.transparent = properties.transparent
		needsUpdate = true
	}

	return needsUpdate
}

function setCommonMaterialProperties(mat: THREE.MeshStandardMaterial): void {
	if (mat.metalness !== 0) {
		mat.metalness = 0
	}

	if (mat.color.getHex() !== 0xffffff) {
		mat.color.set(0xffffff)
	}

	if (mat.roughness !== 1) {
		mat.roughness = 1
	}

	if (!mat.depthTest) {
		mat.depthTest = true
	}

	if (!mat.depthWrite) {
		mat.depthWrite = true
	}
}

function insetSkinGeometryUvs(geometry: THREE.BufferGeometry): void {
	if (geometry.userData.skinUvInsetVersion === SKIN_UV_INSET_VERSION) return

	const uv = geometry.getAttribute('uv')
	if (!uv || uv.itemSize < 2 || uv.count % 4 !== 0) return

	for (let faceStart = 0; faceStart < uv.count; faceStart += 4) {
		let minU = Infinity
		let maxU = -Infinity
		let minV = Infinity
		let maxV = -Infinity

		for (let vertex = faceStart; vertex < faceStart + 4; vertex++) {
			const u = uv.getX(vertex)
			const v = uv.getY(vertex)
			minU = Math.min(minU, u)
			maxU = Math.max(maxU, u)
			minV = Math.min(minV, v)
			maxV = Math.max(maxV, v)
		}

		const insetMinU = (Math.round(minU * SKIN_TEXTURE_SIZE) + 0.5) / SKIN_TEXTURE_SIZE
		const insetMaxU = (Math.round(maxU * SKIN_TEXTURE_SIZE) - 0.5) / SKIN_TEXTURE_SIZE
		const insetMinV = (Math.round(minV * SKIN_TEXTURE_SIZE) + 0.5) / SKIN_TEXTURE_SIZE
		const insetMaxV = (Math.round(maxV * SKIN_TEXTURE_SIZE) - 0.5) / SKIN_TEXTURE_SIZE

		for (let vertex = faceStart; vertex < faceStart + 4; vertex++) {
			const u = uv.getX(vertex)
			const v = uv.getY(vertex)
			const insetU = u === minU ? insetMinU : u === maxU ? insetMaxU : u
			const insetV = v === minV ? insetMinV : v === maxV ? insetMaxV : v
			uv.setXY(vertex, insetU, insetV)
		}
	}

	uv.needsUpdate = true
	geometry.userData.skinUvInsetVersion = SKIN_UV_INSET_VERSION
}

export function applyTexture(model: THREE.Object3D, texture: THREE.Texture): void {
	model.traverse((child) => {
		if ((child as THREE.Mesh).isMesh) {
			const mesh = child as THREE.Mesh
			const isSkinLayer = mesh.name.endsWith('_Layer')
			const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
			const usesSkinTexture = materials.some(
				(mat) => mat instanceof THREE.MeshStandardMaterial && mat.name !== 'cape',
			)

			if (usesSkinTexture) {
				insetSkinGeometryUvs(mesh.geometry)
			}

			materials.forEach((mat: THREE.Material) => {
				if (mat instanceof THREE.MeshStandardMaterial) {
					if (mat.name !== 'cape') {
						const mapNeedsUpdate = applyMap(mat, texture)
						const propertiesNeedUpdate = setShaderMaterialProperties(mat, {
							alphaTest: 0.1,
							flatShading: true,
							side: THREE.FrontSide,
							toneMapped: false,
							transparent: isSkinLayer,
						})

						setCommonMaterialProperties(mat)

						if (mapNeedsUpdate || propertiesNeedUpdate) {
							mat.needsUpdate = true
						}
					}
				}
			})
		}
	})
}

export function applyCapeTexture(
	model: THREE.Object3D,
	texture: THREE.Texture | null,
	transparentTexture?: THREE.Texture,
): void {
	model.traverse((child) => {
		if ((child as THREE.Mesh).isMesh) {
			const mesh = child as THREE.Mesh
			const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

			materials.forEach((mat: THREE.Material) => {
				if (mat instanceof THREE.MeshStandardMaterial) {
					if (mat.name === 'cape') {
						const nextMap = texture || transparentTexture || null
						const mapNeedsUpdate = applyMap(mat, nextMap)
						const propertiesNeedUpdate = setShaderMaterialProperties(mat, {
							alphaTest: 0.1,
							flatShading: true,
							side: THREE.DoubleSide,
							toneMapped: false,
							transparent: !texture || !!transparentTexture,
						})

						setCommonMaterialProperties(mat)

						if (mapNeedsUpdate || propertiesNeedUpdate) {
							mat.needsUpdate = true
						}

						mat.visible = !!texture
					}
				}
			})
		}
	})
}

export function findBodyNode(model: THREE.Object3D): THREE.Object3D | null {
	let bodyNode: THREE.Object3D | null = null

	model.traverse((node) => {
		if (node.name === 'Body') {
			bodyNode = node
		}
	})

	return bodyNode
}

export function createTransparentTexture(): THREE.Texture {
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

export async function setupSkinModel(
	modelUrl: string,
	textureUrl: string,
	capeTextureUrl?: string,
	config: SkinRendererConfig = {},
): Promise<{
	model: THREE.Object3D
	bodyNode: THREE.Object3D | null
}> {
	const [gltf, texture] = await Promise.all([loadModel(modelUrl), loadTexture(textureUrl, config)])

	const model = gltf.scene.clone()
	applyTexture(model, texture)

	if (capeTextureUrl) {
		const capeTexture = await loadTexture(capeTextureUrl, config)
		applyCapeTexture(model, capeTexture)
	}

	const bodyNode = findBodyNode(model)

	return { model, bodyNode }
}

export function disposeCaches(): void {
	Array.from(textureCache.values()).forEach((texture) => {
		texture.dispose()
	})

	textureCache.clear()
	texturePromiseCache.clear()
	modelCache.clear()
	modelPromiseCache.clear()
}
