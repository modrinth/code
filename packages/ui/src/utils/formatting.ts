import type { Labrinth } from '@modrinth/api-client'

export function capitalizeString(name: string) {
	return name ? name.charAt(0).toUpperCase() + name.slice(1) : name
}

export function formatCategory(name: string) {
	if (name === 'modloader') return "Risugami's ModLoader"
	if (name === 'bungeecord') return 'BungeeCord'
	if (name === 'liteloader') return 'LiteLoader'
	if (name === 'neoforge') return 'NeoForge'
	if (name === 'game-mechanics') return 'Game Mechanics'
	if (name === 'worldgen') return 'World Generation'
	if (name === 'core-shaders') return 'Core Shaders'
	if (name === 'gui') return 'GUI'
	if (name === '8x-') return '8x or lower'
	if (name === '512x+') return '512x or higher'
	if (name === 'kitchen-sink') return 'Kitchen Sink'
	if (name === 'path-tracing') return 'Path Tracing'
	if (name === 'pbr') return 'PBR'
	if (name === 'datapack') return 'Data Pack'
	if (name === 'colored-lighting') return 'Colored Lighting'
	if (name === 'optifine') return 'OptiFine'
	if (name === 'bta-babric') return 'BTA (Babric)'
	if (name === 'legacy-fabric') return 'Legacy Fabric'
	if (name === 'java-agent') return 'Java Agent'
	if (name === 'nilloader') return 'NilLoader'
	if (name === 'mrpack') return 'Modpack'
	if (name === 'minecraft') return 'Resource Pack'
	if (name === 'vanilla') return 'Vanilla Shader'
	if (name === 'geyser') return 'Geyser Extension'
	return capitalizeString(name)
}

const mcVersionRegex = /^([0-9]+.[0-9]+)(.[0-9]+)?$/

type VersionRange = {
	major: string
	minor: number[]
}

function groupVersions(versions: string[], consecutive = false) {
	return versions
		.slice()
		.reverse()
		.reduce((ranges: VersionRange[], version: string) => {
			const matchesVersion = version.match(mcVersionRegex)

			if (matchesVersion) {
				const majorVersion = matchesVersion[1]
				const minorVersion = matchesVersion[2]
				const minorNumeric = minorVersion ? parseInt(minorVersion.replace('.', '')) : 0

				const prevInRange = ranges.find(
					(x) => x.major === majorVersion && (!consecutive || x.minor.at(-1) === minorNumeric - 1),
				)
				if (prevInRange) {
					prevInRange.minor.push(minorNumeric)
					return ranges
				}

				return [...ranges, { major: majorVersion, minor: [minorNumeric] }]
			}

			return ranges
		}, [])
		.reverse()
}

function groupConsecutiveIndices(
	versions: string[],
	referenceList: Labrinth.Tags.v2.GameVersion[],
) {
	if (!versions || versions.length === 0) {
		return []
	}

	const referenceMap = new Map<string, number>()
	referenceList.forEach((item, index) => {
		referenceMap.set(item.version, index)
	})

	const sortedList: string[] = versions
		.slice()
		.sort((a, b) => (referenceMap.get(a) ?? 0) - (referenceMap.get(b) ?? 0))

	const ranges: string[] = []
	let start = sortedList[0]
	let previous = sortedList[0]

	for (let i = 1; i < sortedList.length; i++) {
		const current = sortedList[i]
		if ((referenceMap.get(current) ?? 0) !== (referenceMap.get(previous) ?? 0) + 1) {
			ranges.push(validateRange(`${previous}–${start}`))
			start = current
		}
		previous = current
	}

	ranges.push(validateRange(`${previous}–${start}`))

	return ranges
}

function validateRange(range: string): string {
	switch (range) {
		case 'rd-132211–b1.8.1':
			return 'All legacy versions'
		case 'a1.0.4–b1.8.1':
			return 'All alpha and beta versions'
		case 'a1.0.4–a1.2.6':
			return 'All alpha versions'
		case 'b1.0–b1.8.1':
			return 'All beta versions'
		case 'rd-132211–inf20100618':
			return 'All pre-alpha versions'
	}
	const splitRange = range.split('–')
	if (splitRange && splitRange[0] === splitRange[1]) {
		return splitRange[0]
	}
	return range
}

function formatMinecraftMinorVersion(major: string, minor: number): string {
	return minor === 0 ? major : `${major}.${minor}`
}

export function formatVersionsForDisplay(
	gameVersions: string[],
	allGameVersions: Labrinth.Tags.v2.GameVersion[],
) {
	const inputVersions = gameVersions.slice()
	const allVersions = allGameVersions.slice()

	const allSnapshots = allVersions.filter((version) => version.version_type === 'snapshot')
	const allReleases = allVersions.filter((version) => version.version_type === 'release')
	const allLegacy = allVersions.filter(
		(version) => version.version_type !== 'snapshot' && version.version_type !== 'release',
	)

	{
		const indices: Record<string, number> = allVersions.reduce(
			(map, gameVersion, index) => {
				map[gameVersion.version] = index
				return map
			},
			{} as Record<string, number>,
		)
		inputVersions.sort((a, b) => indices[a] - indices[b])
	}

	const releaseVersions = inputVersions.filter((projVer) =>
		allReleases.some((gameVer) => gameVer.version === projVer),
	)

	const dateString = allReleases.find((version) => version.version === releaseVersions[0])?.date

	const latestReleaseVersionDate = dateString ? Date.parse(dateString) : 0
	const latestSnapshot = inputVersions.find((projVer) =>
		allSnapshots.some(
			(gameVer) =>
				gameVer.version === projVer &&
				(!latestReleaseVersionDate || latestReleaseVersionDate < Date.parse(gameVer.date)),
		),
	)

	const allReleasesGrouped = groupVersions(
		allReleases.map((release) => release.version),
		false,
	)
	const projectVersionsGrouped = groupVersions(releaseVersions, true)

	const releaseVersionsAsRanges = projectVersionsGrouped.map(({ major, minor }) => {
		if (minor.length === 1) {
			return formatMinecraftMinorVersion(major, minor[0])
		}

		const range = allReleasesGrouped.find((x) => x.major === major)

		if (range?.minor.every((value, index) => value === minor[index])) {
			return `${major}.x`
		}

		return `${formatMinecraftMinorVersion(major, minor[0])}–${formatMinecraftMinorVersion(major, minor[minor.length - 1])}`
	})

	const legacyVersionsAsRanges = groupConsecutiveIndices(
		inputVersions.filter((projVer) => allLegacy.some((gameVer) => gameVer.version === projVer)),
		allLegacy,
	)

	let output = [...legacyVersionsAsRanges]

	// show all snapshots if there's no release versions
	if (releaseVersionsAsRanges.length === 0) {
		const snapshotVersionsAsRanges = groupConsecutiveIndices(
			inputVersions.filter((projVer) =>
				allSnapshots.some((gameVer) => gameVer.version === projVer),
			),
			allSnapshots,
		)
		output = [...snapshotVersionsAsRanges, ...output]
	} else {
		output = [...releaseVersionsAsRanges, ...output]
	}

	if (latestSnapshot && !output.includes(latestSnapshot)) {
		output = [latestSnapshot, ...output]
	}
	return output
}
