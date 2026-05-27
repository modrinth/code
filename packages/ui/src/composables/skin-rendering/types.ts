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

export interface SkinPreviewFitLock {
	containerSize: {
		width: number
		height: number
	}
	modelCenter: SkinPreviewTuple
	modelSize: SkinPreviewTuple
}

export type SkinPreviewTuple = [number, number, number]
