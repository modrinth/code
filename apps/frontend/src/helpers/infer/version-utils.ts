/**
 * Determines the version type based on the version string.
 */
export function versionType(number: string | null | undefined): 'alpha' | 'beta' | 'release' {
	if (!number) return 'release'
	if (number.includes('alpha')) {
		return 'alpha'
	} else if (
		number.includes('beta') ||
		number.match(/[^A-z](rc)[^A-z]/) || // includes `rc`
		number.match(/[^A-z](pre)[^A-z]/) // includes `pre`
	) {
		return 'beta'
	} else {
		return 'release'
	}
}

/**
 * Extracts version number from a filename.
 */
export function extractVersionFromFilename(filename: string | null | undefined): string | null {
	if (!filename) return null

	// Remove file extension
	let baseName = filename.replace(/\.(zip|jar)$/i, '')

	// Remove explicit MC version markers: mc followed by version (e.g., +mc1.21.11, -mc1.21, _mc1.21.4)
	baseName = baseName.replace(/[+_-]mc\d+\.\d+(?:\.\d+)?/gi, '')

	const versionPatterns = [
		/[_\-\s]v(\d+(?:\.\d+)*)/i, // Match version with 'v' anywhere: "Name-v1.2.3-extra" (less strict)
		/[_\-\s]r(\d+(?:\.\d+)*)/i, // Match version with 'r' anywhere: "Name-r1.2.3-extra" (less strict)
		/[_\-\s](\d+(?:\.\d+)+)$/, // Match version at end after space/separator: "Name 1.2.3"
		/(\d+\.\d+(?:\.\d+)*)/, // Match any version pattern x.x or x.x.x.x...: "Name1.2.3extra"
	]

	for (const pattern of versionPatterns) {
		const match = baseName.match(pattern)
		if (match && match[1]) {
			return match[1]
		}
	}

	return null
}

/**
 * Extracts version details from a filename (public API).
 */
export function extractVersionDetailsFromFilename(filename: string | null | undefined) {
	const versionNum = extractVersionFromFilename(filename)
	return {
		versionNumber: versionNum || undefined,
		versionType: versionType(versionNum),
	}
}
