export interface SkinPreviewAnimationConfig {
	baseAnimation: string
	randomAnimations: string[]
	randomAnimationInterval?: number
	transitionDuration?: number
}

export type SkinPreviewFraming = 'page' | 'modal'

export interface SkinPreviewFitPadding {
	top: number
	right: number
	bottom: number
	left: number
}

export interface SkinPreviewBounds {
	min: SkinPreviewTuple
	max: SkinPreviewTuple
}

export interface SkinPreviewFitLock {
	containerSize: {
		width: number
		height: number
	}
	modelCenter: SkinPreviewTuple
	modelSize: SkinPreviewTuple
	padding: SkinPreviewFitPadding
	rotation: number
	visibleBounds: SkinPreviewBounds
}

export type SkinPreviewTuple = [number, number, number]

export type SkinEarsBodyPart =
	| 'head'
	| 'torso'
	| 'left_arm'
	| 'right_arm'
	| 'left_leg'
	| 'right_leg'

export type SkinEarsTextureSlot = 'skin' | 'displaced_skin' | 'wing' | 'cape'

export interface SkinEarsTexture {
	width: number
	height: number
	/** Base64. */
	rgba: string
}

export interface SkinEarsTexturePair {
	base: SkinEarsTexture
	emissive?: SkinEarsTexture | null
}

export interface SkinEarsQuad {
	part: SkinEarsBodyPart
	texture: SkinEarsTextureSlot
	positions: SkinPreviewTuple[]
	uvs: [number, number][]
}

export interface SkinEarsData {
	skin: SkinEarsTexturePair
	displaced_skin?: SkinEarsTexturePair | null
	wing?: SkinEarsTexturePair | null
	cape?: SkinEarsTexturePair | null
	quads: SkinEarsQuad[]
}

/** Resolves to `null` if the texture has no Ears data. */
export type SkinEarsLoader = (textureSrc: string, slim: boolean) => Promise<SkinEarsData | null>
