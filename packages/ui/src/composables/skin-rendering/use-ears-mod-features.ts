import * as THREE from 'three'

import type { SkinEarsBodyPart, SkinEarsData, SkinEarsTexture, SkinEarsTextureSlot } from './types'

interface TexturePair {
	base: THREE.Texture
	emissive?: THREE.Texture
}

interface Anchor {
	group: THREE.Group
	origin: THREE.Vector3
}

interface EarsModResources {
	featureGroups: THREE.Group[]
	featureMaterials: Set<THREE.Material>
	featureTextures: Set<THREE.Texture>
	emissiveOverlays: THREE.Mesh[]
}

const EARS_REGISTRY_NAME = 'EarsModRegistry'
const EARS_FEATURE_PREFIX = 'EarsModFeature'
const EARS_OVERLAY_PREFIX = 'EarsModEmissiveOverlay'
const QUAD_INDICES = [0, 1, 2, 0, 2, 3]
const TEXTURE_SLOTS: SkinEarsTextureSlot[] = ['skin', 'displaced_skin', 'wing', 'cape']

const BODY_PART_NODES: Record<SkinEarsBodyPart, string> = {
	head: 'Head',
	torso: 'Body',
	left_arm: 'Left_Arm',
	right_arm: 'Right_Arm',
	left_leg: 'Left_Leg',
	right_leg: 'Right_Leg',
}

function createTexture(data: SkinEarsTexture, resources: EarsModResources) {
	const pixels = Uint8Array.from(atob(data.rgba), (char) => char.charCodeAt(0))
	const texture = new THREE.DataTexture(pixels, data.width, data.height, THREE.RGBAFormat)
	texture.colorSpace = THREE.SRGBColorSpace
	texture.flipY = false
	texture.magFilter = THREE.NearestFilter
	texture.minFilter = THREE.NearestFilter
	texture.generateMipmaps = false
	texture.needsUpdate = true
	resources.featureTextures.add(texture)
	return texture
}

function makeFeatureMaterial(texture: THREE.Texture, emissive: boolean) {
	return new THREE.MeshStandardMaterial({
		alphaTest: 0.1,
		color: 0xffffff,
		depthTest: true,
		depthWrite: true,
		emissive: emissive ? 0xffffff : 0x000000,
		emissiveIntensity: emissive ? 1 : 0,
		emissiveMap: emissive ? texture : null,
		flatShading: true,
		map: texture,
		metalness: 0,
		roughness: 1,
		side: THREE.FrontSide,
		toneMapped: false,
		transparent: true,
	})
}

function findBaseMesh(node: THREE.Object3D) {
	const directMesh = node.children.find(
		(child) => (child as THREE.Mesh).isMesh && child.name === `${node.name}_2`,
	) as THREE.Mesh | undefined
	if (directMesh) return directMesh

	let result: THREE.Mesh | null = null
	node.traverse((object) => {
		if (
			!result &&
			(object as THREE.Mesh).isMesh &&
			object.name.endsWith('_2') &&
			!object.name.startsWith(EARS_FEATURE_PREFIX)
		) {
			result = object as THREE.Mesh
		}
	})
	return result
}

function createAnchors(model: THREE.Object3D, resources: EarsModResources) {
	const anchors = new Map<SkinEarsBodyPart, Anchor>()

	for (const [part, nodeName] of Object.entries(BODY_PART_NODES) as [SkinEarsBodyPart, string][]) {
		const node = model.getObjectByName(nodeName)
		if (!node) continue
		const baseMesh = findBaseMesh(node)
		if (!baseMesh?.geometry) continue

		if (!baseMesh.geometry.boundingBox) baseMesh.geometry.computeBoundingBox()
		if (!baseMesh.geometry.boundingBox) continue
		baseMesh.updateMatrix()
		const box = baseMesh.geometry.boundingBox.clone().applyMatrix4(baseMesh.matrix)
		const group = new THREE.Group()
		group.name = `${EARS_FEATURE_PREFIX}:${part}`
		group.scale.setScalar(1 / 16)
		node.add(group)
		resources.featureGroups.push(group)
		anchors.set(part, {
			group,
			origin: new THREE.Vector3(box.max.x * 16, box.min.y * 16, box.min.z * 16),
		})
	}

	return anchors
}

function applySkinTexture(model: THREE.Object3D, texture: THREE.Texture) {
	model.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (!mesh.isMesh || !mesh.material || object.name.startsWith(EARS_FEATURE_PREFIX)) return

		const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		for (const material of materials) {
			if (!(material instanceof THREE.MeshStandardMaterial) || material.name === 'cape') continue
			material.map = texture
			material.needsUpdate = true
		}
	})
}

function addEmissiveBodyOverlays(
	model: THREE.Object3D,
	texture: THREE.Texture,
	resources: EarsModResources,
) {
	const meshes: THREE.Mesh[] = []
	model.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (
			mesh.isMesh &&
			mesh.material &&
			!object.name.startsWith(EARS_FEATURE_PREFIX) &&
			!object.name.startsWith(EARS_OVERLAY_PREFIX)
		) {
			meshes.push(mesh)
		}
	})

	for (const mesh of meshes) {
		const sourceMaterials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		if (
			sourceMaterials.every(
				(material) => !(material instanceof THREE.MeshStandardMaterial) || material.name === 'cape',
			)
		) {
			continue
		}

		const material = makeFeatureMaterial(texture, true)
		resources.featureMaterials.add(material)
		const overlay = new THREE.Mesh(mesh.geometry, material)
		overlay.name = `${EARS_OVERLAY_PREFIX}:${mesh.name}`
		overlay.renderOrder = 2
		mesh.add(overlay)
		resources.emissiveOverlays.push(overlay)
	}
}

function disposeFeatureGroup(group: THREE.Group) {
	group.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (mesh.isMesh) mesh.geometry.dispose()
	})
	group.removeFromParent()
}

export function removeEarsMod(model: THREE.Object3D | null) {
	if (!model) return
	const registry = model.getObjectByName(EARS_REGISTRY_NAME)
	const resources = registry?.userData.earsMod as EarsModResources | undefined
	if (!registry || !resources) return

	for (const overlay of resources.emissiveOverlays) overlay.removeFromParent()
	for (const group of resources.featureGroups) disposeFeatureGroup(group)
	for (const material of resources.featureMaterials) material.dispose()
	for (const texture of resources.featureTextures) texture.dispose()
	registry.removeFromParent()
}

export function isEarsModFeature(object: THREE.Object3D) {
	return object.name.startsWith(EARS_FEATURE_PREFIX)
}

export function applyEarsMod(model: THREE.Object3D, ears: SkinEarsData) {
	removeEarsMod(model)

	const resources: EarsModResources = {
		featureGroups: [],
		featureMaterials: new Set(),
		featureTextures: new Set(),
		emissiveOverlays: [],
	}
	const registry = new THREE.Group()
	registry.name = EARS_REGISTRY_NAME
	registry.userData.earsMod = resources
	model.add(registry)

	const textures = new Map<SkinEarsTextureSlot, TexturePair>()
	for (const slot of TEXTURE_SLOTS) {
		const pair = ears[slot]
		if (!pair) continue
		textures.set(slot, {
			base: createTexture(pair.base, resources),
			emissive: pair.emissive ? createTexture(pair.emissive, resources) : undefined,
		})
	}

	const skinTextures = textures.get('skin')
	if (skinTextures) {
		applySkinTexture(model, skinTextures.base)
		if (skinTextures.emissive) addEmissiveBodyOverlays(model, skinTextures.emissive, resources)
	}

	const anchors = createAnchors(model, resources)
	const materials = new Map<THREE.Texture, THREE.MeshStandardMaterial>()
	const getMaterial = (texture: THREE.Texture, emissive: boolean) => {
		let material = materials.get(texture)
		if (!material) {
			material = makeFeatureMaterial(texture, emissive)
			materials.set(texture, material)
			resources.featureMaterials.add(material)
		}
		return material
	}

	for (const quad of ears.quads) {
		const anchor = anchors.get(quad.part)
		const pair = textures.get(quad.texture)
		if (!anchor || !pair) continue

		const geometry = new THREE.BufferGeometry()
		geometry.setAttribute('position', new THREE.Float32BufferAttribute(quad.positions.flat(), 3))
		geometry.setAttribute('uv', new THREE.Float32BufferAttribute(quad.uvs.flat(), 2))
		geometry.setIndex(QUAD_INDICES)
		geometry.computeVertexNormals()

		const mesh = new THREE.Mesh(geometry, getMaterial(pair.base, false))
		mesh.name = `${EARS_FEATURE_PREFIX}:${quad.texture}`
		mesh.position.copy(anchor.origin)
		mesh.renderOrder = 0
		anchor.group.add(mesh)

		if (pair.emissive) {
			const emissiveMesh = new THREE.Mesh(geometry, getMaterial(pair.emissive, true))
			emissiveMesh.name = `${EARS_FEATURE_PREFIX}:emissive-${quad.texture}`
			emissiveMesh.position.copy(anchor.origin)
			emissiveMesh.renderOrder = 1
			anchor.group.add(emissiveMesh)
		}
	}
}
