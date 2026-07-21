/*
 * Portions of this file are adapted from Ears, Ears2, and the Ears Skin Manipulator:
 * https://git.sleeping.town/exa.mods/Ears
 * https://git.sleeping.town/exa.mods/Ears2
 * Source revisions:
 * - Ears: 5d050d461d207c1a7cda0de11e2562f6581f7a16
 * - Ears2: 8ad9adc213e3b16b1a3306fb4141cfb618c25a6f
 *
 * MIT License
 *
 * Copyright (c) 2020-2026 Exa Skye and contributors
 * Copyright (c) 2021-2026 Exa Skye and contributors
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notices and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

import * as THREE from 'three'

type EarMode =
	| 'NONE'
	| 'ABOVE'
	| 'SIDES'
	| 'BEHIND'
	| 'AROUND'
	| 'FLOPPY'
	| 'CROSS'
	| 'OUT'
	| 'TALL'
	| 'TALL_CROSS'
type EarAnchor = 'CENTER' | 'FRONT' | 'BACK'
type TailMode =
	| 'NONE'
	| 'DOWN'
	| 'BACK'
	| 'UP'
	| 'VERTICAL'
	| 'CROSS'
	| 'CROSS_OVERLAP'
	| 'STAR'
	| 'STAR_OVERLAP'
type WingMode =
	| 'NONE'
	| 'SYMMETRIC_DUAL'
	| 'SYMMETRIC_SINGLE'
	| 'ASYMMETRIC_L'
	| 'ASYMMETRIC_R'
	| 'ASYMMETRIC_DUAL'
	| 'FLAT'
type WingAnimationMode = 'NORMAL' | 'NONE' | 'NO_FLIGHT'
type LegMode = 'PLANTIGRADE' | 'DIGITIGRADE_PARTIAL' | 'DIGITIGRADE_FULL'
type Protrusions =
	| 'NONE'
	| 'HORN'
	| 'CLAWS'
	| 'CLAWS_AND_HORN'
	| 'HALO'
	| 'DOUBLE_HALO'
	| 'CLAWS_AND_HALO'
	| 'CLAWS_AND_DOUBLE_HALO'
type BodyPart = 'HEAD' | 'TORSO' | 'LEFT_ARM' | 'RIGHT_ARM' | 'LEFT_LEG' | 'RIGHT_LEG'
type TextureSource = 'skin' | 'displaced-skin' | 'wing' | 'cape'
type TextureRotation = 'NONE' | 'CW' | 'CCW' | 'UPSIDE_DOWN'
type TextureFlip = 'NONE' | 'HORIZONTAL' | 'VERTICAL' | 'BOTH'

interface EarsFeatures {
	earMode: EarMode
	earAnchor: EarAnchor
	protrusions: Protrusions
	tailMode: TailMode
	tailSegments: number
	tailBends: [number, number, number, number]
	snoutOffset: number
	snoutWidth: number
	snoutHeight: number
	snoutDepth: number
	chestSize: number
	wingMode: WingMode
	wingAnimationMode: WingAnimationMode
	capeEnabled: boolean
	emissive: boolean
	legMode: LegMode
	animateTail: boolean
	swapJacketBackAndTail: boolean
}

interface TexturePair {
	base: THREE.Texture
	emissive?: THREE.Texture
}

interface Anchor {
	group: THREE.Group
	origin: THREE.Vector3
}

interface RenderState {
	anchor: Anchor | null
	matrix: THREE.Matrix4
}

interface EarsModResources {
	featureGroups: THREE.Group[]
	featureMaterials: Set<THREE.Material>
	featureTextures: Set<THREE.Texture>
	emissiveOverlays: THREE.Mesh[]
	pendingImages: HTMLImageElement[]
	objectUrls: Set<string>
	disposed: boolean
}

const EARS_REGISTRY_NAME = 'EarsModRegistry'
const EARS_FEATURE_PREFIX = 'EarsModFeature'
const EARS_OVERLAY_PREFIX = 'EarsModEmissiveOverlay'
const SKIN_SIZE = 64
const EARS_V0_MAGIC = 0x3f23d8
const EARS_V1_MAGIC = 0xea2501
const ALFALFA_MAGIC = 0xea1fa1fa
const UV_TEXEL_INSET = 0.5

const EAR_MODES: EarMode[] = [
	'NONE',
	'ABOVE',
	'SIDES',
	'BEHIND',
	'AROUND',
	'FLOPPY',
	'CROSS',
	'OUT',
	'TALL',
	'TALL_CROSS',
]
const EAR_ANCHORS: EarAnchor[] = ['CENTER', 'FRONT', 'BACK']
const TAIL_MODES: TailMode[] = [
	'NONE',
	'DOWN',
	'BACK',
	'UP',
	'VERTICAL',
	'CROSS',
	'CROSS_OVERLAP',
	'STAR',
	'STAR_OVERLAP',
]
const WING_MODES: WingMode[] = [
	'NONE',
	'SYMMETRIC_DUAL',
	'SYMMETRIC_SINGLE',
	'ASYMMETRIC_L',
	'ASYMMETRIC_R',
	'ASYMMETRIC_DUAL',
	'FLAT',
]
const WING_ANIMATION_MODES: WingAnimationMode[] = ['NORMAL', 'NONE', 'NO_FLIGHT']
const LEG_MODES: LegMode[] = ['PLANTIGRADE', 'DIGITIGRADE_PARTIAL', 'DIGITIGRADE_FULL']
const PROTRUSIONS: Protrusions[] = [
	'NONE',
	'HORN',
	'CLAWS',
	'CLAWS_AND_HORN',
	'HALO',
	'DOUBLE_HALO',
	'CLAWS_AND_HALO',
	'CLAWS_AND_DOUBLE_HALO',
]
const CLAW_PROTRUSIONS = new Set<Protrusions>([
	'CLAWS',
	'CLAWS_AND_HORN',
	'CLAWS_AND_HALO',
	'CLAWS_AND_DOUBLE_HALO',
])

const MAGIC_PIXELS = {
	BLUE: 0x3f23d8,
	GREEN: 0x23d848,
	RED: 0xd82350,
	PURPLE: 0xb923d8,
	CYAN: 0x23d8c6,
	ORANGE: 0xd87823,
	PINK: 0xd823b7,
	PURPLE2: 0xd823ff,
	WHITE: 0xfefdf2,
	GRAY: 0x5e605a,
} as const

const ALFALFA_REGIONS = [
	[8, 0, 24, 8],
	[0, 8, 8, 16],
	[16, 8, 32, 16],
	[4, 16, 12, 20],
	[20, 16, 36, 20],
	[44, 16, 52, 20],
	[0, 20, 56, 32],
	[20, 48, 28, 52],
	[36, 48, 44, 52],
	[16, 52, 48, 64],
] as const

const FORCED_OPAQUE_REGIONS = [
	[8, 0, 24, 8],
	[0, 8, 32, 16],
	[4, 16, 12, 20],
	[20, 16, 36, 20],
	[44, 16, 52, 20],
	[0, 20, 56, 32],
	[20, 48, 28, 52],
	[36, 48, 44, 52],
	[16, 52, 48, 64],
] as const

const FORCED_OPAQUE_REGIONS_WITHOUT_LEG_BOTTOM = [
	[8, 0, 24, 8],
	[0, 8, 32, 16],
	[4, 16, 8, 20],
	[20, 16, 36, 20],
	[44, 16, 52, 20],
	[0, 20, 16, 38],
	[16, 20, 56, 32],
	[20, 48, 24, 52],
	[36, 48, 44, 52],
	[16, 52, 32, 58],
	[32, 52, 48, 64],
] as const

const FORCED_OPAQUE_REGIONS_WITHOUT_LEGS = [
	[8, 0, 24, 8],
	[0, 8, 32, 16],
	[4, 16, 8, 20],
	[20, 16, 36, 20],
	[44, 16, 52, 20],
	[16, 20, 56, 32],
	[20, 48, 24, 52],
	[36, 48, 44, 52],
	[32, 52, 48, 64],
] as const

const LEG_BOTTOM_HALF_REGIONS = [
	[24, 48, 28, 52, true],
	[16, 58, 32, 64, true],
	[8, 16, 12, 20, true],
	[0, 26, 16, 32, true],
	[8, 32, 12, 36, false],
	[0, 42, 16, 48, false],
	[8, 48, 12, 52, false],
	[0, 58, 16, 64, false],
] as const

const LEG_REGIONS = [
	[20, 48, 28, 52, true],
	[16, 52, 32, 64, true],
	[4, 16, 12, 20, true],
	[0, 20, 16, 32, true],
	[4, 32, 12, 36, false],
	[0, 36, 16, 48, false],
	[4, 48, 12, 52, false],
	[0, 52, 16, 64, false],
] as const

const BODY_PART_NODES: Record<BodyPart, string> = {
	HEAD: 'Head',
	TORSO: 'Body',
	LEFT_ARM: 'Left_Arm',
	RIGHT_ARM: 'Right_Arm',
	LEFT_LEG: 'Left_Leg',
	RIGHT_LEG: 'Right_Leg',
}

class BitReader {
	private bitIndex = 0

	constructor(private readonly data: Uint8Array) {}

	get remainingBits() {
		return this.data.length * 8 - this.bitIndex
	}

	read(bits: number) {
		if (this.remainingBits < bits) {
			throw new Error('Unexpected end of Ears data')
		}

		let result = 0
		for (let index = 0; index < bits; index++) {
			const byteIndex = Math.floor(this.bitIndex / 8)
			const shift = 7 - (this.bitIndex % 8)
			result = (result << 1) | ((this.data[byteIndex] >> shift) & 1)
			this.bitIndex++
		}
		return result
	}

	readBoolean() {
		return this.read(1) === 1
	}

	readUnit(bits: number) {
		return this.read(bits) / ((1 << bits) - 1)
	}

	readSignedUnit(bits: number) {
		const negative = this.readBoolean()
		const value = this.read(bits) / ((1 << bits) - 1)
		return negative ? -value : value
	}
}

function pixelOffset(x: number, y: number) {
	return (y * SKIN_SIZE + x) * 4
}

function getPixel(data: Uint8ClampedArray, x: number, y: number) {
	const offset = pixelOffset(x, y)
	const r = data[offset]
	const g = data[offset + 1]
	const b = data[offset + 2]
	const a = data[offset + 3]
	return {
		r,
		g,
		b,
		a,
		rgb: (r << 16) | (g << 8) | b,
	}
}

function getConfigPixel(data: Uint8ClampedArray, index: number) {
	return getPixel(data, index % 4, 32 + Math.floor(index / 4))
}

function getImageDimensions(image: CanvasImageSource) {
	if (image instanceof HTMLImageElement) {
		return [image.naturalWidth, image.naturalHeight] as const
	}
	if (image instanceof HTMLVideoElement) {
		return [image.videoWidth, image.videoHeight] as const
	}
	return [image.width, image.height] as const
}

function readSkinImage(texture: THREE.Texture) {
	const image = texture.image as CanvasImageSource | undefined
	if (!image) return null

	const [width, height] = getImageDimensions(image)
	if (width !== SKIN_SIZE || height !== SKIN_SIZE) return null

	try {
		const canvas = document.createElement('canvas')
		canvas.width = SKIN_SIZE
		canvas.height = SKIN_SIZE
		const context = canvas.getContext('2d', { willReadFrequently: true })
		if (!context) return null

		context.drawImage(image, 0, 0, SKIN_SIZE, SKIN_SIZE)
		return context.getImageData(0, 0, SKIN_SIZE, SKIN_SIZE)
	} catch {
		return null
	}
}

function parseV1Features(data: Uint8ClampedArray): EarsFeatures | null {
	const bytes: number[] = []
	for (let y = 0; y < 4; y++) {
		for (let x = 0; x < 4; x++) {
			if (x === 0 && y === 0) continue
			const pixel = getPixel(data, x, 32 + y)
			bytes.push(pixel.r, pixel.g, pixel.b)
		}
	}

	try {
		const reader = new BitReader(new Uint8Array(bytes))
		const version = reader.read(8)

		const ears = reader.read(6)
		const earMode = ears === 0 ? 'NONE' : (EAR_MODES[Math.floor((ears - 1) / 3) + 1] ?? 'NONE')
		const earAnchor = ears === 0 ? 'CENTER' : (EAR_ANCHORS[(ears - 1) % 3] ?? 'CENTER')
		let protrusionsIndex = reader.read(2)
		const tailIndex = reader.read(3)
		let tailMode = TAIL_MODES[tailIndex] ?? 'NONE'
		let tailSegments = 0
		const tailBends: [number, number, number, number] = [0, 0, 0, 0]

		if (tailIndex !== 0) {
			tailSegments = reader.read(2) + 1
			for (let index = 0; index < tailSegments; index++) {
				tailBends[index] = reader.readSignedUnit(6) * 90
			}
		}

		const snoutWidth = reader.read(3)
		let snoutHeight = 0
		let snoutDepth = 0
		let snoutOffset = 0
		if (snoutWidth > 0) {
			snoutHeight = reader.read(2) + 1
			snoutDepth = reader.read(3) + 1
			snoutOffset = Math.min(reader.read(3), 8 - snoutHeight)
		}

		const chestSize = reader.readUnit(5)
		const wingMode = WING_MODES[reader.read(3)] ?? 'NONE'
		const animateWings = wingMode === 'NONE' ? false : reader.readBoolean()
		const capeEnabled = reader.readBoolean()
		const emissive = reader.readBoolean()
		let wingAnimationMode: WingAnimationMode = animateWings ? 'NORMAL' : 'NONE'
		let legMode: LegMode = 'PLANTIGRADE'
		let animateTail = true
		let swapJacketBackAndTail = false

		if (version >= 1 && tailIndex === 7) {
			tailMode = TAIL_MODES[tailIndex + reader.read(3)] ?? 'NONE'
		}
		if (version >= 2) {
			legMode = LEG_MODES[reader.read(3)] ?? 'PLANTIGRADE'
			if (wingMode !== 'NONE') {
				wingAnimationMode = WING_ANIMATION_MODES[reader.read(3)] ?? wingAnimationMode
			}
			animateTail = reader.readBoolean()
			swapJacketBackAndTail = reader.readBoolean()
		}
		if (version >= 3) {
			protrusionsIndex |= reader.read(2) << 2
		}

		return {
			earMode,
			earAnchor,
			protrusions: PROTRUSIONS[protrusionsIndex] ?? 'NONE',
			tailMode,
			tailSegments,
			tailBends,
			snoutOffset,
			snoutWidth,
			snoutHeight,
			snoutDepth,
			chestSize,
			wingMode,
			wingAnimationMode,
			capeEnabled,
			emissive,
			legMode,
			animateTail,
			swapJacketBackAndTail,
		}
	} catch {
		return null
	}
}

function pixelValueToUnit(value: number) {
	if (value === 0) return 0
	let shifted = value - 128
	shifted += shifted < 0 ? -1 : 1
	return shifted / 128
}

function parseV0Features(data: Uint8ClampedArray): EarsFeatures {
	const earModeByPixel = new Map<number, EarMode>([
		[MAGIC_PIXELS.RED, 'NONE'],
		[MAGIC_PIXELS.BLUE, 'ABOVE'],
		[MAGIC_PIXELS.GREEN, 'SIDES'],
		[MAGIC_PIXELS.PURPLE, 'BEHIND'],
		[MAGIC_PIXELS.CYAN, 'AROUND'],
		[MAGIC_PIXELS.ORANGE, 'FLOPPY'],
		[MAGIC_PIXELS.PINK, 'CROSS'],
		[MAGIC_PIXELS.PURPLE2, 'OUT'],
		[MAGIC_PIXELS.WHITE, 'TALL'],
		[MAGIC_PIXELS.GRAY, 'TALL_CROSS'],
	])
	const earAnchorByPixel = new Map<number, EarAnchor>([
		[MAGIC_PIXELS.BLUE, 'CENTER'],
		[MAGIC_PIXELS.GREEN, 'FRONT'],
		[MAGIC_PIXELS.RED, 'BACK'],
	])
	const tailModeByPixel = new Map<number, TailMode>([
		[MAGIC_PIXELS.RED, 'NONE'],
		[MAGIC_PIXELS.BLUE, 'DOWN'],
		[MAGIC_PIXELS.GREEN, 'BACK'],
		[MAGIC_PIXELS.PURPLE, 'UP'],
		[MAGIC_PIXELS.ORANGE, 'VERTICAL'],
		[MAGIC_PIXELS.PINK, 'CROSS'],
		[MAGIC_PIXELS.PURPLE2, 'CROSS_OVERLAP'],
		[MAGIC_PIXELS.WHITE, 'STAR'],
		[MAGIC_PIXELS.GRAY, 'STAR_OVERLAP'],
	])
	const wingModeByPixel = new Map<number, WingMode>([
		[MAGIC_PIXELS.BLUE, 'NONE'],
		[MAGIC_PIXELS.RED, 'NONE'],
		[MAGIC_PIXELS.PINK, 'SYMMETRIC_DUAL'],
		[MAGIC_PIXELS.GREEN, 'SYMMETRIC_SINGLE'],
		[MAGIC_PIXELS.CYAN, 'ASYMMETRIC_L'],
		[MAGIC_PIXELS.ORANGE, 'ASYMMETRIC_R'],
		[MAGIC_PIXELS.PURPLE, 'ASYMMETRIC_DUAL'],
		[MAGIC_PIXELS.PURPLE2, 'FLAT'],
	])
	const protrusionsByPixel = new Map<number, Protrusions>([
		[MAGIC_PIXELS.BLUE, 'NONE'],
		[MAGIC_PIXELS.RED, 'NONE'],
		[MAGIC_PIXELS.GREEN, 'CLAWS'],
		[MAGIC_PIXELS.PURPLE, 'HORN'],
		[MAGIC_PIXELS.CYAN, 'CLAWS_AND_HORN'],
		[MAGIC_PIXELS.WHITE, 'HALO'],
		[MAGIC_PIXELS.GRAY, 'DOUBLE_HALO'],
		[MAGIC_PIXELS.PURPLE2, 'CLAWS_AND_HALO'],
		[MAGIC_PIXELS.PINK, 'CLAWS_AND_DOUBLE_HALO'],
	])
	const wingAnimationModeByPixel = new Map<number, WingAnimationMode>([
		[MAGIC_PIXELS.BLUE, 'NORMAL'],
		[MAGIC_PIXELS.RED, 'NONE'],
		[MAGIC_PIXELS.GREEN, 'NO_FLIGHT'],
	])
	const legModeByPixel = new Map<number, LegMode>([
		[MAGIC_PIXELS.BLUE, 'PLANTIGRADE'],
		[MAGIC_PIXELS.GREEN, 'DIGITIGRADE_PARTIAL'],
		[MAGIC_PIXELS.PINK, 'DIGITIGRADE_FULL'],
	])

	const earMode = earModeByPixel.get(getConfigPixel(data, 1).rgb) ?? 'NONE'
	const earAnchor = earAnchorByPixel.get(getConfigPixel(data, 2).rgb) ?? 'CENTER'
	const protrusions = protrusionsByPixel.get(getConfigPixel(data, 3).rgb) ?? 'NONE'
	const tailMode = tailModeByPixel.get(getConfigPixel(data, 4).rgb) ?? 'NONE'
	const tailPixel = getConfigPixel(data, 5)
	const tailBends: [number, number, number, number] = [0, 0, 0, 0]
	let tailSegments = 0

	if (tailPixel.rgb !== MAGIC_PIXELS.BLUE) {
		tailBends[0] = pixelValueToUnit(255 - tailPixel.a) * 90
		tailBends[1] = pixelValueToUnit(tailPixel.r) * 90
		tailBends[2] = pixelValueToUnit(tailPixel.g) * 90
		tailBends[3] = pixelValueToUnit(tailPixel.b) * 90
		tailSegments = 1
		if (tailBends[1] !== 0) tailSegments = 2
		if (tailBends[2] !== 0) tailSegments = 3
		if (tailBends[3] !== 0) tailSegments = 4
	}

	const snoutPixel = getConfigPixel(data, 6)
	const etcPixel = getConfigPixel(data, 7)
	let snoutWidth = 0
	let snoutHeight = 0
	let snoutDepth = 0
	let snoutOffset = 0
	if (snoutPixel.rgb !== MAGIC_PIXELS.BLUE) {
		snoutWidth = Math.min(snoutPixel.r, 7)
		snoutHeight = Math.min(snoutPixel.g, 4)
		snoutDepth = Math.min(snoutPixel.b, 8)
		snoutOffset = Math.min(etcPixel.g, 8 - snoutHeight)
	}

	const etcIsDefault = etcPixel.rgb === MAGIC_PIXELS.BLUE
	const chestSize = etcIsDefault ? 0 : Math.min(etcPixel.r / 128, 1)
	const capeEnabled = !etcIsDefault && (etcPixel.b & 16) !== 0
	const bitflags = getConfigPixel(data, 12).rgb
	const bitflagsAreDefault = bitflags === MAGIC_PIXELS.BLUE

	return {
		earMode,
		earAnchor,
		protrusions,
		tailMode,
		tailSegments,
		tailBends,
		snoutOffset,
		snoutWidth,
		snoutHeight,
		snoutDepth,
		chestSize,
		wingMode: wingModeByPixel.get(getConfigPixel(data, 8).rgb) ?? 'NONE',
		wingAnimationMode: wingAnimationModeByPixel.get(getConfigPixel(data, 9).rgb) ?? 'NORMAL',
		capeEnabled,
		emissive: getConfigPixel(data, 10).rgb === MAGIC_PIXELS.ORANGE,
		legMode: legModeByPixel.get(getConfigPixel(data, 11).rgb) ?? 'PLANTIGRADE',
		animateTail: bitflagsAreDefault || (bitflags & 1) !== 0,
		swapJacketBackAndTail: !bitflagsAreDefault && (bitflags & 2) !== 0,
	}
}

function parseFeatures(data: Uint8ClampedArray) {
	const magic = getPixel(data, 0, 32).rgb
	if (magic === EARS_V0_MAGIC) return parseV0Features(data)
	if (magic === EARS_V1_MAGIC) return parseV1Features(data)
	return null
}

function parseAlfalfa(data: Uint8ClampedArray) {
	let packed = 0n
	let index = 0n

	for (const [x1, y1, x2, y2] of ALFALFA_REGIONS) {
		for (let x = x1; x < x2; x++) {
			for (let y = y1; y < y2; y++) {
				const alpha = data[pixelOffset(x, y) + 3]
				if (alpha === 0) continue
				const value = 0x7f - (alpha & 0x7f)
				packed |= BigInt(value) << (index * 7n)
				index++
			}
		}
	}

	if (packed === 0n) return new Map<string, Uint8Array>()

	let hex = packed.toString(16)
	if (hex.length % 2 !== 0) hex = `0${hex}`
	const bytes = new Uint8Array(hex.length / 2 + 1)
	for (let byteIndex = 0; byteIndex < hex.length / 2; byteIndex++) {
		bytes[byteIndex + 1] = Number.parseInt(hex.slice(byteIndex * 2, byteIndex * 2 + 2), 16)
	}

	let offset = 1
	const readByte = () => {
		if (offset >= bytes.length) throw new Error('Unexpected end of Alfalfa data')
		return bytes[offset++]
	}

	try {
		const magic = ((readByte() << 24) | (readByte() << 16) | (readByte() << 8) | readByte()) >>> 0
		if (magic !== ALFALFA_MAGIC || readByte() !== 1) {
			return new Map<string, Uint8Array>()
		}

		const entries = new Map<string, Uint8Array>()
		const predefinedKeys = ['END', 'wing', 'erase', 'cape']
		while (offset < bytes.length) {
			const first = readByte()
			let key: string
			if (first < 64) {
				key = predefinedKeys[first] ?? `!unk${first}`
			} else {
				const chars = [first]
				while ((chars[chars.length - 1] & 0x80) === 0) {
					chars.push(readByte())
				}
				chars[chars.length - 1] &= 0x7f
				key = String.fromCharCode(...chars)
			}
			if (key === 'END') break

			const value: number[] = []
			let length: number
			do {
				length = readByte()
				for (let valueIndex = 0; valueIndex < length; valueIndex++) {
					value.push(readByte())
				}
			} while (length === 255)
			entries.set(key, new Uint8Array(value))
		}
		return entries
	} catch {
		return new Map<string, Uint8Array>()
	}
}

function applyEraseData(data: Uint8ClampedArray, eraseData: Uint8Array | undefined) {
	if (!eraseData) return

	const reader = new BitReader(eraseData)
	try {
		while (reader.remainingBits >= 22) {
			const x = reader.read(6)
			const y = reader.read(6)
			const width = reader.read(5) + 1
			const height = reader.read(5) + 1
			for (let pixelX = x; pixelX < Math.min(x + width, SKIN_SIZE); pixelX++) {
				for (let pixelY = y; pixelY < Math.min(y + height, SKIN_SIZE); pixelY++) {
					const offset = pixelOffset(pixelX, pixelY)
					data[offset] = 0
					data[offset + 1] = 0
					data[offset + 2] = 0
					data[offset + 3] = 0
				}
			}
		}
	} catch {
		return
	}
}

function forceOpaqueSkinRegions(data: Uint8ClampedArray, legMode: LegMode) {
	const regions =
		legMode === 'DIGITIGRADE_FULL'
			? FORCED_OPAQUE_REGIONS_WITHOUT_LEGS
			: legMode === 'DIGITIGRADE_PARTIAL'
				? FORCED_OPAQUE_REGIONS_WITHOUT_LEG_BOTTOM
				: FORCED_OPAQUE_REGIONS
	for (const [x1, y1, x2, y2] of regions) {
		for (let x = x1; x < x2; x++) {
			for (let y = y1; y < y2; y++) {
				data[pixelOffset(x, y) + 3] = 255
			}
		}
	}
}

function displaceDigitigradeLegPixels(data: Uint8ClampedArray, legMode: LegMode) {
	if (legMode === 'PLANTIGRADE') return null

	const displaced = new Uint8ClampedArray(data.length)
	const regions = legMode === 'DIGITIGRADE_FULL' ? LEG_REGIONS : LEG_BOTTOM_HALF_REGIONS
	for (const [x1, y1, x2, y2, forceOpaque] of regions) {
		for (let x = x1; x < x2; x++) {
			for (let y = y1; y < y2; y++) {
				const offset = pixelOffset(x, y)
				displaced.set(data.subarray(offset, offset + 4), offset)
				if (forceOpaque) displaced[offset + 3] = 255
				data.fill(0, offset, offset + 4)
			}
		}
	}
	return displaced
}

function swapJacketBackAndTailPixels(data: Uint8ClampedArray) {
	const tail = new Uint8ClampedArray(8 * 12 * 4)
	for (let x = 0; x < 8; x++) {
		for (let y = 0; y < 12; y++) {
			const tailOffset = pixelOffset(56 + x, 16 + y)
			const savedOffset = (y * 8 + x) * 4
			tail.set(data.subarray(tailOffset, tailOffset + 4), savedOffset)

			const jacketOffset = pixelOffset(32 + x, 36 + (11 - y))
			data.set(data.subarray(jacketOffset, jacketOffset + 4), tailOffset)
		}
	}
	for (let x = 0; x < 8; x++) {
		for (let y = 0; y < 12; y++) {
			const savedOffset = (y * 8 + x) * 4
			data.set(tail.subarray(savedOffset, savedOffset + 4), pixelOffset(32 + x, 36 + y))
		}
	}
}

function getEmissivePalette(data: Uint8ClampedArray) {
	const palette = new Set<number>()
	for (let x = 52; x < 56; x++) {
		for (let y = 32; y < 36; y++) {
			const pixel = getPixel(data, x, y)
			if (pixel.a > 0) palette.add(pixel.rgb)
		}
	}
	return palette
}

function splitEmissivePixels(data: Uint8ClampedArray, palette: Set<number>) {
	const base = new Uint8ClampedArray(data)
	const emissive = new Uint8ClampedArray(data)

	for (let offset = 0; offset < data.length; offset += 4) {
		const rgb = (data[offset] << 16) | (data[offset + 1] << 8) | data[offset + 2]
		if (palette.has(rgb)) {
			base[offset + 3] = 0
		} else {
			emissive[offset + 3] = 0
		}
	}

	return { base, emissive }
}

function createCanvasTexture(data: Uint8ClampedArray, width: number, height: number) {
	const canvas = document.createElement('canvas')
	canvas.width = width
	canvas.height = height
	const context = canvas.getContext('2d')
	if (!context) throw new Error('Could not create Ears texture canvas')
	context.putImageData(new ImageData(data, width, height), 0, 0)

	const texture = new THREE.CanvasTexture(canvas)
	texture.colorSpace = THREE.SRGBColorSpace
	texture.flipY = false
	texture.magFilter = THREE.NearestFilter
	texture.minFilter = THREE.NearestFilter
	texture.generateMipmaps = false
	return texture
}

function readPngDimensions(data: Uint8Array) {
	if (
		data.length < 24 ||
		data[0] !== 0x89 ||
		data[1] !== 0x50 ||
		data[2] !== 0x4e ||
		data[3] !== 0x47
	) {
		return null
	}

	const view = new DataView(data.buffer, data.byteOffset, data.byteLength)
	return [view.getUint32(16), view.getUint32(20)] as const
}

function createEmbeddedTexture(
	data: Uint8Array,
	targetWidth: number,
	targetHeight: number,
	resources: EarsModResources,
	palette?: Set<number>,
	legacyWing = false,
): TexturePair {
	const baseCanvas = document.createElement('canvas')
	baseCanvas.width = targetWidth
	baseCanvas.height = targetHeight
	const baseTexture = new THREE.CanvasTexture(baseCanvas)
	baseTexture.colorSpace = THREE.SRGBColorSpace
	baseTexture.flipY = false
	baseTexture.magFilter = THREE.NearestFilter
	baseTexture.minFilter = THREE.NearestFilter
	baseTexture.generateMipmaps = false
	resources.featureTextures.add(baseTexture)

	let emissiveCanvas: HTMLCanvasElement | undefined
	let emissiveTexture: THREE.CanvasTexture | undefined
	if (palette?.size) {
		emissiveCanvas = document.createElement('canvas')
		emissiveCanvas.width = targetWidth
		emissiveCanvas.height = targetHeight
		emissiveTexture = new THREE.CanvasTexture(emissiveCanvas)
		emissiveTexture.colorSpace = THREE.SRGBColorSpace
		emissiveTexture.flipY = false
		emissiveTexture.magFilter = THREE.NearestFilter
		emissiveTexture.minFilter = THREE.NearestFilter
		emissiveTexture.generateMipmaps = false
		resources.featureTextures.add(emissiveTexture)
	}

	const blob = new Blob([data as BlobPart], { type: 'image/png' })
	const objectUrl = URL.createObjectURL(blob)
	const image = new Image()
	resources.objectUrls.add(objectUrl)
	resources.pendingImages.push(image)

	image.onload = () => {
		resources.objectUrls.delete(objectUrl)
		URL.revokeObjectURL(objectUrl)
		if (resources.disposed) return

		const rawCanvas = document.createElement('canvas')
		rawCanvas.width = targetWidth
		rawCanvas.height = targetHeight
		const rawContext = rawCanvas.getContext('2d', { willReadFrequently: true })
		const baseContext = baseCanvas.getContext('2d')
		if (!rawContext || !baseContext) return

		rawContext.clearRect(0, 0, targetWidth, targetHeight)
		rawContext.drawImage(image, 0, legacyWing ? 2 : 0)
		if (!palette?.size || !emissiveCanvas || !emissiveTexture) {
			baseContext.drawImage(rawCanvas, 0, 0)
			baseTexture.needsUpdate = true
			return
		}

		const rawData = rawContext.getImageData(0, 0, targetWidth, targetHeight)
		const split = splitEmissivePixels(rawData.data, palette)
		baseContext.putImageData(new ImageData(split.base, targetWidth, targetHeight), 0, 0)
		emissiveCanvas
			.getContext('2d')
			?.putImageData(new ImageData(split.emissive, targetWidth, targetHeight), 0, 0)
		baseTexture.needsUpdate = true
		emissiveTexture.needsUpdate = true
	}
	image.onerror = () => {
		resources.objectUrls.delete(objectUrl)
		URL.revokeObjectURL(objectUrl)
	}
	image.src = objectUrl

	return {
		base: baseTexture,
		emissive: emissiveTexture,
	}
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

function flipHorizontally(flip: TextureFlip): TextureFlip {
	switch (flip) {
		case 'NONE':
			return 'HORIZONTAL'
		case 'HORIZONTAL':
			return 'NONE'
		case 'VERTICAL':
			return 'BOTH'
		case 'BOTH':
			return 'VERTICAL'
	}
}

function calculateUvs(
	u: number,
	v: number,
	width: number,
	height: number,
	rotation: TextureRotation,
	flip: TextureFlip,
	textureWidth: number,
	textureHeight: number,
) {
	const transposed = rotation === 'CW' || rotation === 'CCW'
	let minU = (u + UV_TEXEL_INSET) / textureWidth
	let minV = (v + UV_TEXEL_INSET) / textureHeight
	let maxU = (u + (transposed ? height : width) - UV_TEXEL_INSET) / textureWidth
	let maxV = (v + (transposed ? width : height) - UV_TEXEL_INSET) / textureHeight
	let resolvedFlip = flip

	if (transposed) {
		if (resolvedFlip === 'HORIZONTAL') resolvedFlip = 'VERTICAL'
		else if (resolvedFlip === 'VERTICAL') resolvedFlip = 'HORIZONTAL'
	}
	if (resolvedFlip === 'HORIZONTAL' || resolvedFlip === 'BOTH') {
		;[minU, maxU] = [maxU, minU]
	}
	if (resolvedFlip === 'VERTICAL' || resolvedFlip === 'BOTH') {
		;[minV, maxV] = [maxV, minV]
	}

	const uvs = [
		[minU, maxV],
		[maxU, maxV],
		[maxU, minV],
		[minU, minV],
	]
	if (rotation === 'CW') {
		uvs.unshift(uvs.pop()!)
	} else if (rotation === 'CCW') {
		uvs.push(uvs.shift()!)
	} else if (rotation === 'UPSIDE_DOWN') {
		uvs.push(uvs.shift()!, uvs.shift()!)
	}
	return uvs
}

class EarsGeometryBuilder {
	private readonly materials = new Map<string, THREE.MeshStandardMaterial>()
	private readonly stack: RenderState[] = []
	private state: RenderState = {
		anchor: null,
		matrix: new THREE.Matrix4(),
	}
	private textureSource: TextureSource = 'skin'

	constructor(
		private readonly anchors: Map<BodyPart, Anchor>,
		private readonly textures: Map<TextureSource, TexturePair>,
		private readonly resources: EarsModResources,
	) {}

	push() {
		this.stack.push({
			anchor: this.state.anchor,
			matrix: this.state.matrix.clone(),
		})
	}

	pop() {
		const state = this.stack.pop()
		if (state) this.state = state
	}

	anchorTo(part: BodyPart) {
		const anchor = this.anchors.get(part) ?? null
		this.state.anchor = anchor
		this.state.matrix.identity()
		if (anchor) {
			this.state.matrix.makeTranslation(anchor.origin.x, anchor.origin.y, anchor.origin.z)
		}
	}

	bind(source: TextureSource) {
		this.textureSource = source
	}

	translate(x: number, y: number, z: number) {
		this.state.matrix.multiply(new THREE.Matrix4().makeTranslation(-x, -y, z))
	}

	rotate(angle: number, x: number, y: number, z: number) {
		const axis = new THREE.Vector3(-x, -y, z).normalize()
		this.state.matrix.multiply(
			new THREE.Matrix4().makeRotationAxis(axis, THREE.MathUtils.degToRad(angle)),
		)
	}

	scale(x: number, y: number, z: number) {
		this.state.matrix.multiply(new THREE.Matrix4().makeScale(x, y, z))
	}

	front(
		u: number,
		v: number,
		width: number,
		height: number,
		rotation: TextureRotation = 'NONE',
		flip: TextureFlip = 'NONE',
		grow = 0,
	) {
		this.quad(u, v, width, height, rotation, flip, grow, false, 0, 0, 0)
	}

	frontSkew(
		u: number,
		v: number,
		width: number,
		height: number,
		xSkew: number,
		ySkew: number,
		zSkew: number,
		rotation: TextureRotation = 'NONE',
		flip: TextureFlip = 'NONE',
		grow = 0,
	) {
		this.quad(u, v, width, height, rotation, flip, grow, false, xSkew, ySkew, zSkew)
	}

	back(
		u: number,
		v: number,
		width: number,
		height: number,
		rotation: TextureRotation = 'NONE',
		flip: TextureFlip = 'NONE',
		grow = 0,
	) {
		this.quad(u, v, width, height, rotation, flipHorizontally(flip), grow, true, 0, 0, 0)
	}

	doubleSided(
		u: number,
		v: number,
		width: number,
		height: number,
		rotation: TextureRotation = 'NONE',
		flip: TextureFlip = 'NONE',
		grow = 0,
	) {
		this.front(u, v, width, height, rotation, flip, grow)
		this.back(u, v, width, height, rotation, flipHorizontally(flip), grow)
	}

	private getMaterial(texture: THREE.Texture, emissive: boolean) {
		const key = `${texture.uuid}:${emissive}`
		let material = this.materials.get(key)
		if (!material) {
			material = makeFeatureMaterial(texture, emissive)
			this.materials.set(key, material)
			this.resources.featureMaterials.add(material)
		}
		return material
	}

	private quad(
		u: number,
		v: number,
		width: number,
		height: number,
		rotation: TextureRotation,
		flip: TextureFlip,
		grow: number,
		back: boolean,
		xSkew: number,
		ySkew: number,
		zSkew: number,
	) {
		const anchor = this.state.anchor
		const textures = this.textures.get(this.textureSource)
		if (!anchor || !textures) return

		const skinTexture = this.textureSource === 'skin' || this.textureSource === 'displaced-skin'
		const textureWidth = skinTexture ? 64 : 20
		const textureHeight = skinTexture ? 64 : 16
		const uvs = calculateUvs(u, v, width, height, rotation, flip, textureWidth, textureHeight)
		const positions = back
			? [
					grow + xSkew,
					grow + ySkew,
					-zSkew,
					-width - grow + xSkew,
					grow,
					-zSkew,
					-width - grow - xSkew,
					-height - grow,
					zSkew,
					grow - xSkew,
					-height - grow + ySkew,
					zSkew,
				]
			: [
					grow - xSkew,
					-height - grow + ySkew,
					zSkew,
					-width - grow - xSkew,
					-height - grow - ySkew,
					zSkew,
					-width - grow + xSkew,
					grow - ySkew,
					-zSkew,
					grow + xSkew,
					grow + ySkew,
					-zSkew,
				]
		const orderedUvs = back ? [uvs[3], uvs[2], uvs[1], uvs[0]] : [uvs[0], uvs[1], uvs[2], uvs[3]]
		const geometry = new THREE.BufferGeometry()
		geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3))
		geometry.setAttribute('uv', new THREE.Float32BufferAttribute(orderedUvs.flat(), 2))
		geometry.setIndex([0, 1, 2, 0, 2, 3])
		geometry.computeVertexNormals()

		const mesh = new THREE.Mesh(geometry, this.getMaterial(textures.base, false))
		mesh.name = `${EARS_FEATURE_PREFIX}:${this.textureSource}`
		mesh.matrix.copy(this.state.matrix)
		mesh.matrixAutoUpdate = false
		mesh.renderOrder = 0
		anchor.group.add(mesh)

		if (textures.emissive) {
			const emissiveMesh = new THREE.Mesh(geometry, this.getMaterial(textures.emissive, true))
			emissiveMesh.name = `${EARS_FEATURE_PREFIX}:emissive-${this.textureSource}`
			emissiveMesh.matrix.copy(this.state.matrix)
			emissiveMesh.matrixAutoUpdate = false
			emissiveMesh.renderOrder = 1
			anchor.group.add(emissiveMesh)
		}
	}
}

function applyEarAnchor(builder: EarsGeometryBuilder, anchor: EarAnchor) {
	if (anchor === 'CENTER') builder.translate(0, 0, 4)
	else if (anchor === 'BACK') builder.translate(0, 0, 8)
}

function renderEars(builder: EarsGeometryBuilder, features: EarsFeatures) {
	const mode = features.earMode
	if (mode === 'NONE') return

	if (mode === 'ABOVE' || mode === 'AROUND') {
		builder.push()
		builder.anchorTo('HEAD')
		applyEarAnchor(builder, features.earAnchor)
		builder.push()
		builder.translate(-4, -16, 0)
		builder.front(24, 0, 16, 8)
		builder.back(56, 28, 16, 8, 'CW')
		builder.pop()
		if (mode === 'AROUND') {
			builder.translate(-4, -8, 0)
			builder.front(36, 16, 4, 8, 'CW')
			builder.back(12, 16, 4, 8, 'CW')
			builder.translate(12, 0, 0)
			builder.front(36, 32, 4, 8, 'CW')
			builder.back(12, 32, 4, 8, 'CW')
		}
		builder.pop()
		return
	}

	if (mode === 'SIDES') {
		builder.push()
		builder.anchorTo('HEAD')
		applyEarAnchor(builder, features.earAnchor)
		builder.translate(-8, -8, 0)
		builder.front(24, 0, 8, 8)
		builder.back(56, 28, 8, 8, 'CW')
		builder.translate(16, 0, 0)
		builder.front(32, 0, 8, 8)
		builder.back(56, 36, 8, 8, 'CW')
		builder.pop()
		return
	}

	if (mode === 'BEHIND') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.rotate(90, 0, 1, 0)
		builder.translate(-16, -8, 0)
		builder.front(24, 0, 8, 8)
		builder.back(56, 28, 8, 8, 'CW')
		builder.rotate(180, 0, 1, 0)
		builder.translate(-8, 0, -8)
		builder.front(32, 0, 8, 8)
		builder.back(56, 36, 8, 8, 'CW')
		builder.pop()
		return
	}

	if (mode === 'FLOPPY') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.rotate(90, 0, 1, 0)
		builder.translate(-8, -7, 0)
		builder.rotate(-30, 1, 0, 0)
		builder.front(24, 0, 8, 8)
		builder.back(56, 28, 8, 8, 'CW')
		builder.pop()

		builder.push()
		builder.anchorTo('HEAD')
		builder.rotate(-90, 0, 1, 0)
		builder.translate(0, -7, -8)
		builder.rotate(-30, 1, 0, 0)
		builder.front(32, 0, 8, 8)
		builder.back(56, 36, 8, 8, 'CW')
		builder.pop()
		return
	}

	if (mode === 'CROSS') {
		builder.push()
		builder.anchorTo('HEAD')
		applyEarAnchor(builder, features.earAnchor)
		builder.translate(4, -16, 0)
		builder.push()
		builder.rotate(45, 0, 1, 0)
		builder.translate(-4, 0, 0)
		builder.front(24, 0, 8, 8)
		builder.back(56, 28, 8, 8, 'CW')
		builder.pop()
		builder.push()
		builder.rotate(-45, 0, 1, 0)
		builder.translate(-4, 0, 0)
		builder.front(32, 0, 8, 8)
		builder.back(56, 36, 8, 8, 'CW')
		builder.pop()
		builder.pop()
		return
	}

	if (mode === 'OUT') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.rotate(90, 0, 1, 0)
		if (features.earAnchor === 'BACK') builder.translate(-16, -8, 0)
		else if (features.earAnchor === 'CENTER') builder.translate(-8, -16, 0)
		else builder.translate(0, -8, 0)
		builder.front(24, 0, 8, 8)
		builder.back(56, 28, 8, 8, 'CW')
		builder.rotate(180, 0, 1, 0)
		builder.translate(-8, 0, -8)
		builder.front(32, 0, 8, 8)
		builder.back(56, 36, 8, 8, 'CW')
		builder.pop()
		return
	}

	if (mode === 'TALL') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.translate(0, -8, 0)
		applyEarAnchor(builder, features.earAnchor)
		const angle = -6
		builder.rotate(angle / 3, 1, 0, 0)
		builder.translate(0, -4, 0)
		builder.front(24, 0, 8, 4, 'CW')
		builder.back(56, 40, 8, 4)
		builder.rotate(angle, 1, 0, 0)
		builder.translate(0, -4, 0)
		builder.front(28, 0, 8, 4, 'CW')
		builder.back(56, 36, 8, 4)
		builder.rotate(angle / 2, 1, 0, 0)
		builder.translate(0, -4, 0)
		builder.front(32, 0, 8, 4, 'CW')
		builder.back(56, 32, 8, 4)
		builder.rotate(angle, 1, 0, 0)
		builder.translate(0, -4, 0)
		builder.front(36, 0, 8, 4, 'CW')
		builder.back(56, 28, 8, 4)
		builder.pop()
		return
	}

	builder.push()
	builder.anchorTo('HEAD')
	applyEarAnchor(builder, features.earAnchor)
	builder.translate(4, -24, 0)
	builder.push()
	builder.rotate(45, 0, 1, 0)
	builder.translate(-4, 0, 0)
	builder.front(24, 0, 8, 16, 'CW')
	builder.back(56, 28, 8, 16)
	builder.pop()
	builder.push()
	builder.rotate(-45, 0, 1, 0)
	builder.translate(-4, 0, 0)
	builder.front(24, 0, 8, 16, 'CW')
	builder.back(56, 28, 8, 16)
	builder.pop()
	builder.pop()
}

function renderTail(builder: EarsGeometryBuilder, features: EarsFeatures) {
	const mode = features.tailMode
	if (mode === 'NONE') return

	let angle = 0
	if (mode === 'DOWN') angle = 30
	else if (
		mode === 'BACK' ||
		mode === 'CROSS' ||
		mode === 'CROSS_OVERLAP' ||
		mode === 'STAR' ||
		mode === 'STAR_OVERLAP'
	) {
		angle = features.tailBends[0] !== 0 ? 90 : 80
	} else if (mode === 'UP') {
		angle = 130
	}

	const vertical = mode === 'VERTICAL'
	const baseAngle = features.tailBends[0]
	builder.push()
	builder.anchorTo('TORSO')
	builder.translate(0, -2, 4)
	builder.rotate(angle, 1, 0, 0)
	if (vertical) {
		builder.translate(4, 0, 0)
		builder.rotate(90, 0, 0, 1)
		if (baseAngle < 0) {
			builder.translate(4, 0, 0)
			builder.rotate(baseAngle, 0, 1, 0)
			builder.translate(-4, 0, 0)
		}
		builder.translate(-4, 0, 0)
		if (baseAngle > 0) builder.rotate(baseAngle, 0, 1, 0)
		builder.rotate(90, 1, 0, 0)
	}

	const segments = Math.max(features.tailSegments, 1)
	const segmentHeight = Math.floor(12 / segments)
	const angles = vertical
		? [0, features.tailBends[1], features.tailBends[2], features.tailBends[3]]
		: features.tailBends

	for (let index = 0; index < segments; index++) {
		const overlap = index === 0 || (mode !== 'CROSS_OVERLAP' && mode !== 'STAR_OVERLAP') ? 0 : 4
		builder.rotate(angles[index], 1, 0, 0)
		builder.doubleSided(56, 16 + index * segmentHeight, 8, segmentHeight, 'NONE', 'HORIZONTAL')
		if (mode === 'CROSS' || mode === 'CROSS_OVERLAP') {
			builder.push()
			builder.translate(4, 0, 0)
			builder.rotate(90, 0, 1, 0)
			builder.translate(-4, -overlap, 0)
			builder.doubleSided(
				56,
				16 + index * segmentHeight - overlap,
				8,
				segmentHeight + overlap,
				'NONE',
				'HORIZONTAL',
			)
			builder.pop()
		} else if (mode === 'STAR' || mode === 'STAR_OVERLAP') {
			for (let side = 0; side < 3; side++) {
				builder.push()
				builder.translate(4, 0, 0)
				builder.rotate(45 * (side + 1), 0, 1, 0)
				builder.translate(-4, -overlap, 0)
				builder.doubleSided(
					56,
					16 + index * segmentHeight - overlap,
					8,
					segmentHeight + overlap,
					'NONE',
					'HORIZONTAL',
				)
				builder.pop()
			}
		}
		builder.translate(0, segmentHeight, 0)
	}
	builder.pop()
}

function renderProtrusions(builder: EarsGeometryBuilder, features: EarsFeatures, slim: boolean) {
	if (CLAW_PROTRUSIONS.has(features.protrusions)) {
		const legOffset =
			features.legMode === 'DIGITIGRADE_FULL'
				? -1.5
				: features.legMode === 'DIGITIGRADE_PARTIAL'
					? -1
					: 0
		builder.push()
		builder.anchorTo('LEFT_LEG')
		builder.translate(0, 0, -4 + legOffset)
		builder.rotate(90, 1, 0, 0)
		builder.doubleSided(16, 48, 4, 4, 'NONE', 'HORIZONTAL')
		builder.pop()

		builder.push()
		builder.anchorTo('RIGHT_LEG')
		builder.translate(0, 0, -4 + legOffset)
		builder.rotate(90, 1, 0, 0)
		builder.doubleSided(0, 16, 4, 4, 'NONE', 'HORIZONTAL')
		builder.pop()

		builder.push()
		builder.anchorTo('LEFT_ARM')
		builder.rotate(90, 0, 1, 0)
		builder.translate(-4, 0, slim ? 3 : 4)
		builder.doubleSided(44, 48, 4, 4, 'UPSIDE_DOWN', 'HORIZONTAL')
		builder.pop()

		builder.push()
		builder.anchorTo('RIGHT_ARM')
		builder.rotate(90, 0, 1, 0)
		builder.translate(-4, 0, 0)
		builder.doubleSided(52, 16, 4, 4, 'UPSIDE_DOWN')
		builder.pop()
	}

	if (features.protrusions === 'HORN' || features.protrusions === 'CLAWS_AND_HORN') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.translate(0, -8, 0)
		builder.rotate(25, 1, 0, 0)
		builder.translate(0, -8, 0)
		builder.doubleSided(56, 0, 8, 8)
		builder.pop()
	}

	if (features.protrusions === 'HALO' || features.protrusions === 'CLAWS_AND_HALO') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.translate(0, -12, 0)
		builder.rotate(90, 1, 0, 0)
		builder.doubleSided(56, 0, 8, 8)
		builder.pop()
	}

	if (features.protrusions === 'DOUBLE_HALO' || features.protrusions === 'CLAWS_AND_DOUBLE_HALO') {
		builder.push()
		builder.anchorTo('HEAD')
		builder.translate(0, -10, 0)
		builder.rotate(90, 1, 0, 0)
		for (let index = 0; index < 2; index++) {
			builder.translate(4, 4, 0)
			builder.rotate(45, 0, 0, 1)
			builder.translate(-4, -4, 2)
			builder.doubleSided(56, 0, 8, 8)
		}
		builder.pop()
	}
}

function drawDigitigradeLeg(
	builder: EarsGeometryBuilder,
	u: number,
	v: number,
	grow: number,
	bottom: boolean,
	mend: boolean,
) {
	const width = 4
	const height = 12
	const depth = 4
	const doubleGrow = grow * 2
	const skew = 1
	const vOffset = bottom ? 10 : 4

	builder.push()
	builder.translate(width / 2, height / 2, depth / 2)
	builder.scale(
		(width + doubleGrow) / width,
		(height + doubleGrow) / height,
		(depth + doubleGrow) / depth,
	)
	builder.translate(-width / 2, -height / 2, -depth / 2)

	builder.push()
	builder.translate(0, bottom ? -6 : -12, 0)
	if (!mend && bottom) {
		builder.push()
		builder.translate(0, 2, 0)
		builder.frontSkew(u + 4, v + vOffset + 2, 4, 4, 0, 0, -skew)
		builder.pop()
	} else {
		builder.frontSkew(u + 4, v + vOffset, 4, 6, 0, 0, -skew)
	}
	if (mend && !bottom) {
		builder.push()
		builder.translate(0, 6, 0)
		builder.frontSkew(u + 4, v + vOffset + 6, 4, 2, 0, 0, skew)
		builder.pop()
	}

	builder.rotate(-90, 0, 1, 0)
	builder.translate(0, 0, -4)
	if (!mend && bottom) {
		builder.push()
		builder.translate(0, 2, 0)
		builder.frontSkew(u + 8, v + vOffset + 2, 4, 4, -skew, 0, 0)
		builder.pop()
	} else {
		builder.frontSkew(u + 8, v + vOffset, 4, 6, -skew, 0, 0)
	}
	if (mend && !bottom) {
		builder.push()
		builder.translate(0, 6, 0)
		builder.frontSkew(u + 8, v + vOffset + 6, 4, 2, skew, 0, 0)
		builder.pop()
	}

	builder.rotate(-90, 0, 1, 0)
	builder.translate(0, 0, -4)
	if (!mend && bottom) {
		builder.push()
		builder.translate(0, 2, 0)
		builder.frontSkew(u + 12, v + vOffset + 2, 4, 4, 0, 0, skew)
		builder.pop()
	} else {
		builder.frontSkew(u + 12, v + vOffset, 4, 6, 0, 0, skew)
	}
	if (mend && !bottom) {
		builder.push()
		builder.translate(0, 6, 0)
		builder.frontSkew(u + 12, v + vOffset + 5, 4, 2, 0, 0, -skew)
		builder.pop()
	}

	builder.rotate(-90, 0, 1, 0)
	builder.translate(0, 0, -4)
	if (!mend && bottom) {
		builder.push()
		builder.translate(0, 2, 0)
		builder.frontSkew(u, v + vOffset + 2, 4, 4, skew, 0, 0)
		builder.pop()
	} else {
		builder.frontSkew(u, v + vOffset, 4, 6, skew, 0, 0)
	}
	if (mend && !bottom) {
		builder.push()
		builder.translate(0, 6, 0)
		builder.frontSkew(u, v + vOffset + 6, 4, 2, -skew, 0, 0)
		builder.pop()
	}
	builder.pop()

	builder.push()
	builder.rotate(90, 1, 0, 0)
	if (bottom) {
		builder.push()
		builder.translate(0, -skew, 0)
		builder.front(u + 8, v, 4, 4, 'NONE', 'VERTICAL')
		builder.pop()
	} else {
		builder.push()
		builder.translate(0, skew, 12)
		builder.back(u + 8, v, 4, 4, 'NONE', 'VERTICAL')
		builder.pop()
	}
	if (mend && bottom) {
		builder.translate(0, 0, 6)
		builder.front(u + 4, v + vOffset, 4, 1)
		builder.translate(0, 4, 0)
		builder.back(u + 12, v + vOffset, 4, 1)
	}
	builder.pop()
	builder.pop()
}

function renderDigitigradeLegs(builder: EarsGeometryBuilder, features: EarsFeatures) {
	if (features.legMode === 'PLANTIGRADE') return

	const full = features.legMode === 'DIGITIGRADE_FULL'
	builder.bind('displaced-skin')
	for (let layer = 0; layer < 2; layer++) {
		const grow = layer === 0 ? 0 : 0.25

		builder.push()
		builder.anchorTo('LEFT_LEG')
		if (full) builder.translate(0, 0, -0.5)
		if (layer === 1) builder.translate(0, 0.5, 0)
		if (full) drawDigitigradeLeg(builder, layer === 0 ? 16 : 0, 48, grow, false, true)
		drawDigitigradeLeg(builder, layer === 0 ? 16 : 0, 48, grow, true, !full)
		builder.pop()

		builder.push()
		builder.anchorTo('RIGHT_LEG')
		if (full) builder.translate(0, 0, -0.5)
		if (layer === 1) builder.translate(0, 0.5, 0)
		if (full) drawDigitigradeLeg(builder, 0, layer === 0 ? 16 : 32, grow, false, true)
		drawDigitigradeLeg(builder, 0, layer === 0 ? 16 : 32, grow, true, !full)
		builder.pop()
	}
	builder.bind('skin')
}

function renderSnout(builder: EarsGeometryBuilder, features: EarsFeatures) {
	const { snoutOffset, snoutWidth, snoutHeight, snoutDepth } = features
	if (snoutWidth <= 0 || snoutHeight <= 0 || snoutDepth <= 0) return

	builder.push()
	builder.anchorTo('HEAD')
	builder.translate((8 - snoutWidth) / 2, -(snoutOffset + snoutHeight), -snoutDepth)
	builder.doubleSided(0, 2, snoutWidth, snoutHeight)

	builder.push()
	builder.rotate(-90, 1, 0, 0)
	builder.translate(0, -1, 0)
	builder.doubleSided(0, 1, snoutWidth, 1)
	for (let index = 0; index < snoutDepth - 1; index++) {
		builder.translate(0, -1, 0)
		builder.doubleSided(0, 0, snoutWidth, 1)
	}
	builder.pop()

	builder.push()
	builder.translate(0, snoutHeight, 0)
	builder.rotate(90, 1, 0, 0)
	builder.doubleSided(0, 2 + snoutHeight, snoutWidth, 1)
	for (let index = 0; index < snoutDepth - 1; index++) {
		builder.translate(0, 1, 0)
		builder.doubleSided(0, 3 + snoutHeight, snoutWidth, 1)
	}
	builder.pop()

	builder.push()
	builder.rotate(90, 0, 1, 0)
	builder.push()
	builder.translate(-1, 0, 0)
	builder.doubleSided(7, 0, 1, snoutHeight)
	for (let index = 0; index < snoutDepth - 1; index++) {
		builder.translate(-1, 0, 0)
		builder.doubleSided(7, 4, 1, snoutHeight)
	}
	builder.pop()
	builder.push()
	builder.translate(-1, 0, snoutWidth)
	builder.doubleSided(7, 0, 1, snoutHeight)
	for (let index = 0; index < snoutDepth - 1; index++) {
		builder.translate(-1, 0, 0)
		builder.doubleSided(7, 4, 1, snoutHeight)
	}
	builder.pop()
	builder.pop()
	builder.pop()
}

function renderChest(builder: EarsGeometryBuilder, features: EarsFeatures) {
	if (features.chestSize <= 0) return

	for (const layer of [0, 1]) {
		builder.push()
		builder.anchorTo('TORSO')
		builder.translate(0, -10, 0)
		builder.rotate(-features.chestSize * 45, 1, 0, 0)
		if (layer === 0) {
			builder.doubleSided(20, 22, 8, 4)
		} else {
			builder.push()
			builder.translate(4, 2, 0)
			builder.scale(8.5 / 8, 4.5 / 4, 1)
			builder.translate(-4, -2, -0.25)
			builder.doubleSided(0, 48, 4, 4)
			builder.translate(4, 0, 0)
			builder.doubleSided(12, 48, 4, 4)
			builder.pop()
		}

		builder.push()
		builder.translate(0, 4, 0)
		builder.rotate(90, 1, 0, 0)
		if (layer === 0) {
			builder.doubleSided(56, 44, 8, 4)
		} else {
			builder.translate(0, 0, -0.25)
			builder.doubleSided(28, 48, 8, 4, 'NONE', 'NONE', 0.25)
		}
		builder.pop()

		builder.push()
		builder.rotate(90, 0, 1, 0)
		builder.translate(-4, 0, 0.01)
		if (layer === 0) {
			builder.doubleSided(60, 48, 4, 4)
		} else {
			builder.translate(0, 0, -0.25)
			builder.doubleSided(48, 48, 4, 4, 'NONE', 'NONE', 0.25)
			builder.translate(0, 0, 0.25)
		}
		builder.translate(0, 0, 7.98)
		builder.rotate(180, 0, 1, 0)
		builder.translate(-4, 0, 0)
		if (layer === 0) {
			builder.doubleSided(60, 48, 4, 4, 'NONE', 'HORIZONTAL')
		} else {
			builder.translate(0, 0, -0.25)
			builder.doubleSided(48, 48, 4, 4, 'NONE', 'HORIZONTAL', 0.25)
		}
		builder.pop()
		builder.pop()
	}
}

function renderWings(builder: EarsGeometryBuilder, features: EarsFeatures) {
	const mode = features.wingMode
	if (mode === 'NONE') return

	const wiggle = features.wingAnimationMode !== 'NONE' ? Math.sin(8 / 12) * 2 : 0
	builder.push()
	builder.anchorTo('TORSO')
	builder.bind('wing')
	builder.translate(2, -14, 4)

	if (mode === 'SYMMETRIC_DUAL' || mode === 'ASYMMETRIC_R') {
		builder.push()
		builder.rotate(-120 + wiggle, 0, 1, 0)
		builder.doubleSided(0, 0, 20, 16)
		builder.pop()
	}
	if (mode === 'SYMMETRIC_DUAL' || mode === 'ASYMMETRIC_L') {
		builder.translate(4, 0, 0)
		builder.push()
		builder.rotate(-60 - wiggle, 0, 1, 0)
		builder.doubleSided(0, 0, 20, 16)
		builder.pop()
	}
	if (mode === 'SYMMETRIC_SINGLE') {
		builder.translate(2, 0, 0)
		builder.push()
		builder.rotate(-90 + wiggle, 0, 1, 0)
		builder.doubleSided(0, 0, 20, 16)
		builder.pop()
	}
	if (mode === 'ASYMMETRIC_DUAL') {
		builder.push()
		builder.rotate(-120 + wiggle, 0, 1, 0)
		builder.doubleSided(0, 0, 10, 16)
		builder.pop()
		builder.translate(4, 0, 0)
		builder.push()
		builder.rotate(-60 - wiggle, 0, 1, 0)
		builder.doubleSided(10, 0, 10, 16)
		builder.pop()
	}
	if (mode === 'FLAT') {
		builder.translate(-8, 0, 0.75)
		builder.doubleSided(0, 0, 20, 16)
	}
	builder.bind('skin')
	builder.pop()
}

function renderCape(builder: EarsGeometryBuilder, features: EarsFeatures) {
	if (!features.capeEnabled) return

	builder.push()
	builder.anchorTo('TORSO')
	builder.translate(4, -12, 5)
	builder.rotate(6, 1, 0, 0)
	builder.rotate(180, 0, 1, 0)
	builder.bind('cape')
	builder.translate(-5, 0, 0)
	builder.doubleSided(0, 0, 10, 16)

	builder.push()
	builder.translate(10, 0, 1)
	builder.rotate(90, 0, 1, 0)
	builder.doubleSided(9, 0, 1, 16, 'NONE', 'HORIZONTAL')
	builder.rotate(90, 0, 1, 0)
	builder.doubleSided(10, 0, 10, 16)
	builder.translate(10, 0, 1)
	builder.rotate(90, 0, 1, 0)
	builder.doubleSided(0, 0, 1, 16, 'NONE', 'HORIZONTAL')
	builder.pop()

	builder.rotate(90, 1, 0, 0)
	builder.doubleSided(0, 0, 10, 1, 'NONE', 'VERTICAL')
	builder.translate(0, 0, -16)
	builder.doubleSided(0, 15, 10, 1, 'NONE', 'VERTICAL')
	builder.bind('skin')
	builder.pop()
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
	const anchors = new Map<BodyPart, Anchor>()

	for (const [part, nodeName] of Object.entries(BODY_PART_NODES) as [BodyPart, string][]) {
		const node = model.getObjectByName(nodeName)
		if (!node) continue
		const baseMesh = findBaseMesh(node)
		if (!baseMesh?.geometry) continue

		if (!baseMesh.geometry.boundingBox) baseMesh.geometry.computeBoundingBox()
		if (!baseMesh.geometry.boundingBox) continue
		baseMesh.updateMatrix()
		const box = baseMesh.geometry.boundingBox.clone().applyMatrix4(baseMesh.matrix)
		const group = new THREE.Group()
		group.name = `${EARS_FEATURE_PREFIX}:${part.toLowerCase()}`
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

function isSlimModel(model: THREE.Object3D) {
	const arm = model.getObjectByName('Right_Arm')
	const mesh = arm ? findBaseMesh(arm) : null
	if (!mesh?.geometry) return false
	if (!mesh.geometry.boundingBox) mesh.geometry.computeBoundingBox()
	const box = mesh.geometry.boundingBox
	return box ? box.max.x - box.min.x < 0.24 : false
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

	resources.disposed = true
	for (const image of resources.pendingImages) {
		image.onload = null
		image.onerror = null
		image.src = ''
	}
	for (const objectUrl of resources.objectUrls) URL.revokeObjectURL(objectUrl)
	for (const overlay of resources.emissiveOverlays) overlay.removeFromParent()
	for (const group of resources.featureGroups) disposeFeatureGroup(group)
	for (const material of resources.featureMaterials) material.dispose()
	for (const texture of resources.featureTextures) texture.dispose()
	registry.removeFromParent()
}

export function applyEarsMod(model: THREE.Object3D, sourceTexture: THREE.Texture, enabled = true) {
	removeEarsMod(model)

	const sourceImage = readSkinImage(sourceTexture)
	if (!sourceImage) return false
	const features = parseFeatures(sourceImage.data)
	if (!features) return false
	if (!enabled) return true

	const resources: EarsModResources = {
		featureGroups: [],
		featureMaterials: new Set(),
		featureTextures: new Set(),
		emissiveOverlays: [],
		pendingImages: [],
		objectUrls: new Set(),
		disposed: false,
	}
	const registry = new THREE.Group()
	registry.name = EARS_REGISTRY_NAME
	registry.userData.earsMod = resources
	model.add(registry)

	const alfalfa = parseAlfalfa(sourceImage.data)
	const processedData = new Uint8ClampedArray(sourceImage.data)
	applyEraseData(processedData, alfalfa.get('erase'))
	const displacedData = displaceDigitigradeLegPixels(processedData, features.legMode)
	if (features.swapJacketBackAndTail) swapJacketBackAndTailPixels(processedData)
	forceOpaqueSkinRegions(processedData, features.legMode)

	const palette = features.emissive ? getEmissivePalette(processedData) : new Set<number>()
	if (palette.size === 0) features.emissive = false
	const skinPixels = features.emissive
		? splitEmissivePixels(processedData, palette)
		: { base: processedData, emissive: undefined }
	const skinTextures: TexturePair = {
		base: createCanvasTexture(skinPixels.base, SKIN_SIZE, SKIN_SIZE),
		emissive: skinPixels.emissive
			? createCanvasTexture(skinPixels.emissive, SKIN_SIZE, SKIN_SIZE)
			: undefined,
	}
	resources.featureTextures.add(skinTextures.base)
	if (skinTextures.emissive) resources.featureTextures.add(skinTextures.emissive)
	applySkinTexture(model, skinTextures.base)
	if (skinTextures.emissive) addEmissiveBodyOverlays(model, skinTextures.emissive, resources)

	const textures = new Map<TextureSource, TexturePair>([['skin', skinTextures]])
	if (displacedData) {
		const displacedPixels = features.emissive
			? splitEmissivePixels(displacedData, palette)
			: { base: displacedData, emissive: undefined }
		const displacedTextures: TexturePair = {
			base: createCanvasTexture(displacedPixels.base, SKIN_SIZE, SKIN_SIZE),
			emissive: displacedPixels.emissive
				? createCanvasTexture(displacedPixels.emissive, SKIN_SIZE, SKIN_SIZE)
				: undefined,
		}
		resources.featureTextures.add(displacedTextures.base)
		if (displacedTextures.emissive) resources.featureTextures.add(displacedTextures.emissive)
		textures.set('displaced-skin', displacedTextures)
	}
	const wingData = alfalfa.get('wing')
	const wingDimensions = wingData ? readPngDimensions(wingData) : null
	if (
		features.wingMode !== 'NONE' &&
		wingData &&
		wingDimensions &&
		((wingDimensions[0] === 20 && wingDimensions[1] === 16) ||
			(wingDimensions[0] === 12 && wingDimensions[1] === 12))
	) {
		textures.set(
			'wing',
			createEmbeddedTexture(
				wingData,
				20,
				16,
				resources,
				features.emissive ? palette : undefined,
				wingDimensions[0] === 12,
			),
		)
	} else {
		features.wingMode = 'NONE'
	}

	const capeData = alfalfa.get('cape')
	const capeDimensions = capeData ? readPngDimensions(capeData) : null
	if (features.capeEnabled && capeData && capeDimensions?.[0] === 20 && capeDimensions[1] === 16) {
		textures.set('cape', createEmbeddedTexture(capeData, 20, 16, resources))
	} else {
		features.capeEnabled = false
	}

	const anchors = createAnchors(model, resources)
	const builder = new EarsGeometryBuilder(anchors, textures, resources)
	renderEars(builder, features)
	renderTail(builder, features)
	renderProtrusions(builder, features, isSlimModel(model))
	renderSnout(builder, features)
	renderChest(builder, features)
	renderWings(builder, features)
	renderCape(builder, features)
	renderDigitigradeLegs(builder, features)
	return true
}
