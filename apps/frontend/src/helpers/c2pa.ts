import type { C2paSdk } from '@contentauth/c2pa-web'
import { createC2pa, isSupportedReaderFormat } from '@contentauth/c2pa-web/inline'

// https://cv.iptc.org/newscodes/digitalsourcetype/
// only detecting generative AI types
const AI_DIGITAL_SOURCE_TYPES = [
	'http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia',
	'http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicData',
	'http://cv.iptc.org/newscodes/digitalsourcetype/compositeWithTrainedAlgorithmicMedia',
	'http://cv.iptc.org/newscodes/digitalsourcetype/compositeSynthetic',
]

let c2paInstance: Promise<C2paSdk> | null = null

function getC2pa() {
	if (!c2paInstance) {
		c2paInstance = createC2pa()
	}
	return c2paInstance
}

// reference: https://github.com/contentauth/c2pa-js/blob/main/packages/c2pa-web/README.md
export async function fileDeclaresAi(file: File): Promise<boolean> {
	try {
		if (!isSupportedReaderFormat(file.type)) {
			return false
		}

		const c2pa = await getC2pa()

		const reader = await c2pa.reader.fromBlob(file.type, file)
		const manifestStore = await reader.manifestStore()

		await reader.free()

		const manifestJson = JSON.stringify(manifestStore)
		return AI_DIGITAL_SOURCE_TYPES.some((type) => manifestJson.includes(type))
	} catch {
		return false
	}
}
