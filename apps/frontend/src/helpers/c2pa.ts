import type { C2paSdk } from '@contentauth/c2pa-web'

// https://cv.iptc.org/newscodes/digitalsourcetype/
// only detecting generative AI types
const AI_DIGITAL_SOURCE_TYPES = [
	'http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia',
	'http://cv.iptc.org/newscodes/digitalsourcetype/compositeWithTrainedAlgorithmicMedia',
	'http://cv.iptc.org/newscodes/digitalsourcetype/compositeSynthetic',
]

let c2paInstance: Promise<C2paSdk> | null = null

// reference: https://github.com/contentauth/c2pa-js/blob/main/packages/c2pa-web/README.md
export async function fileDeclaresAi(file: File): Promise<boolean> {
	if (!import.meta.client) {
		return false
	}

	try {
		const { createC2pa, isSupportedReaderFormat } = await import('@contentauth/c2pa-web')

		if (!isSupportedReaderFormat(file.type)) {
			return false
		}

		if (!c2paInstance) {
			c2paInstance = import('@contentauth/c2pa-web/resources/c2pa.wasm?url').then((wasm) =>
				createC2pa({ wasmSrc: wasm.default }),
			)
		}

		const c2pa = await c2paInstance

		const reader = await c2pa.reader.fromBlob(file.type, file)
		const manifestStore = await reader.manifestStore()

		await reader.free()

		const manifestJson = JSON.stringify(manifestStore)
		return AI_DIGITAL_SOURCE_TYPES.some((type) => manifestJson.includes(type))
	} catch {
		return false
	}
}
