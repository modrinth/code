import * as THREE from 'three'

export interface SkinRendererConfig {
	textureColorSpace?: THREE.ColorSpace
	textureFlipY?: boolean
	textureMagFilter?: THREE.MagnificationTextureFilter
	textureMinFilter?: THREE.MinificationTextureFilter
}

/** The caller owns the returned texture and must dispose it. */
export async function loadTexture(
	textureUrl: string,
	config: SkinRendererConfig = {},
): Promise<THREE.Texture> {
	const texture = await new THREE.TextureLoader().loadAsync(textureUrl)
	texture.colorSpace = config.textureColorSpace ?? THREE.SRGBColorSpace
	texture.flipY = config.textureFlipY ?? false
	texture.magFilter = config.textureMagFilter ?? THREE.NearestFilter
	texture.minFilter = config.textureMinFilter ?? THREE.NearestFilter
	return texture
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

export function applyTexture(model: THREE.Object3D, texture: THREE.Texture): void {
	model.traverse((child) => {
		if ((child as THREE.Mesh).isMesh) {
			const mesh = child as THREE.Mesh
			const isSkinLayer = mesh.name.endsWith('_Layer')
			const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

			materials.forEach((mat: THREE.Material) => {
				if (mat instanceof THREE.MeshStandardMaterial) {
					if (mat.name !== 'cape') {
						const mapNeedsUpdate = applyMap(mat, texture)
						const propertiesNeedUpdate = setShaderMaterialProperties(mat, {
							alphaTest: 0.1,
							flatShading: true,
							side: isSkinLayer ? THREE.DoubleSide : THREE.FrontSide,
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
