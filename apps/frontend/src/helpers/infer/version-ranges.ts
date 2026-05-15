import { satisfies } from 'semver'

/**
 * Returns game versions that match a semver range or array of ranges.
 */
export function getGameVersionsMatchingSemverRange(
	range: string | string[] | undefined,
	gameVersions: string[],
): string[] {
	if (!range) {
		return []
	}
	const ranges = Array.isArray(range) ? range : [range]
	// Normalize ranges: strip trailing hyphens from version numbers used by Fabric for prerelease matching (e.g., ">=1.21.11-" -> ">=1.21.11")
	const normalizedRanges = ranges.map((r) => r.replace(/(\d)-(\s|$)/g, '$1$2'))
	return gameVersions.filter((version) => {
		const semverVersion = version.split('.').length === 2 ? `${version}.0` : version // add patch version if missing (e.g. 1.16 -> 1.16.0)
		return normalizedRanges.some((v) => satisfies(semverVersion, v))
	})
}

/**
 * Returns game versions that match a Maven-style version range.
 */
export function getGameVersionsMatchingMavenRange(
	range: string | undefined,
	gameVersions: string[],
	processor: (version: string) => string | null = (v) => v,
): string[] {
	if (!range) {
		return []
	}
	const ranges = []

	while (range.startsWith('[') || range.startsWith('(')) {
		let index = range.indexOf(')')
		const index2 = range.indexOf(']')
		if (index === -1 || (index2 !== -1 && index2 < index)) {
			index = index2
		}
		if (index === -1) break
		ranges.push(range.substring(0, index + 1))
		range = range.substring(index + 1).trim()
		if (range.startsWith(',')) {
			range = range.substring(1).trim()
		}
	}

	if (range) {
		ranges.push(range)
	}

	const LESS_THAN_EQUAL = /^\(,(.*)]$/
	const LESS_THAN = /^\(,(.*)\)$/
	const EQUAL = /^\[(.*)]$/
	const GREATER_THAN_EQUAL = /^\[(.*),\)$/
	const GREATER_THAN = /^\((.*),\)$/
	const BETWEEN = /^\((.*),(.*)\)$/
	const BETWEEN_EQUAL = /^\[(.*),(.*)]$/
	const BETWEEN_LESS_THAN_EQUAL = /^\((.*),(.*)]$/
	const BETWEEN_GREATER_THAN_EQUAL = /^\[(.*),(.*)\)$/

	const semverRanges = []

	for (const range of ranges) {
		let result
		let version1
		let version2
		if ((result = range.match(LESS_THAN_EQUAL)) && (version1 = processor(result[1]))) {
			semverRanges.push(`<=${version1}`)
		} else if ((result = range.match(LESS_THAN)) && (version1 = processor(result[1]))) {
			semverRanges.push(`<${version1}`)
		} else if ((result = range.match(EQUAL)) && (version1 = processor(result[1]))) {
			semverRanges.push(`${version1}`)
		} else if ((result = range.match(GREATER_THAN_EQUAL)) && (version1 = processor(result[1]))) {
			semverRanges.push(`>=${version1}`)
		} else if ((result = range.match(GREATER_THAN)) && (version1 = processor(result[1]))) {
			semverRanges.push(`>${version1}`)
		} else if (
			(result = range.match(BETWEEN)) &&
			(version1 = processor(result[1])) &&
			(version2 = processor(result[2]))
		) {
			semverRanges.push(`>${version1} <${version2}`)
		} else if (
			(result = range.match(BETWEEN_EQUAL)) &&
			(version1 = processor(result[1])) &&
			(version2 = processor(result[2]))
		) {
			semverRanges.push(`>=${version1} <=${version2}`)
		} else if (
			(result = range.match(BETWEEN_LESS_THAN_EQUAL)) &&
			(version1 = processor(result[1])) &&
			(version2 = processor(result[2]))
		) {
			semverRanges.push(`>${version1} <=${version2}`)
		} else if (
			(result = range.match(BETWEEN_GREATER_THAN_EQUAL)) &&
			(version1 = processor(result[1])) &&
			(version2 = processor(result[2]))
		) {
			semverRanges.push(`>=${version1} <${version2}`)
		}
	}
	return getGameVersionsMatchingSemverRange(semverRanges, gameVersions)
}
