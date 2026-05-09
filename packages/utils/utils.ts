// noinspection JSUnusedGlobalSymbols

// @ts-nocheck

import dayjs from 'dayjs'

export const external = (cosmetics) => (cosmetics.externalLinksNewTab ? '_blank' : '')

// Only use on the complete list of versions for a project,
// partial lists will generate the wrong version slugs
export const computeVersions = (versions, members) => {
	const visitedVersions = []
	const returnVersions = []
	const authorMembers = {}

	for (const version of versions.sort(
		(a, b) => dayjs(a.date_published) - dayjs(b.date_published),
	)) {
		if (visitedVersions.includes(version.version_number)) {
			visitedVersions.push(version.version_number)
			version.displayUrlEnding = version.id
		} else {
			visitedVersions.push(version.version_number)
			version.displayUrlEnding = version.version_number
		}
		version.primaryFile = version.files.find((file) => file.primary) ?? version.files[0]

		if (!version.primaryFile) {
			version.primaryFile = {
				hashes: {
					sha1: '',
					sha512: '',
				},
				url: '#',
				filename: 'unknown',
				primary: false,
				size: 0,
				file_type: null,
			}
		}

		version.author = authorMembers[version.author_id]
		if (!version.author) {
			version.author = members.find((x) => x.user.id === version.author_id)
			authorMembers[version.author_id] = version.author
		}

		returnVersions.push(version)
	}

	return returnVersions
		.reverse()
		.map((version, index) => {
			const nextVersion = returnVersions[index + 1]
			if (nextVersion && version.changelog && nextVersion.changelog === version.changelog) {
				return { duplicate: true, ...version }
			}
			return { duplicate: false, ...version }
		})
		.sort((a, b) => dayjs(b.date_published) - dayjs(a.date_published))
}

const SERVER_HEADER_ORDER = [
	'minecraft_server_features',
	'minecraft_server_gameplay',
	'minecraft_server_meta',
	'minecraft_server_community',
]

export const sortedCategories = (tags, formatCategoryName, locale) => {
	return tags.categories.slice().sort((a, b) => {
		const headerCompare = a.header.localeCompare(b.header)
		if (headerCompare !== 0) {
			const aServerIdx = SERVER_HEADER_ORDER.indexOf(a.header)
			const bServerIdx = SERVER_HEADER_ORDER.indexOf(b.header)
			if (aServerIdx !== -1 && bServerIdx !== -1) {
				return aServerIdx - bServerIdx
			}

			return headerCompare
		}

		if (a.header === 'performance impact' && b.header === 'performance impact') {
			const x = ['potato', 'low', 'medium', 'high', 'screenshot']
			return x.indexOf(a.name) - x.indexOf(b.name)
		}

		if (a.name === 'pokemon') return -1
		if (b.name === 'pokemon') return 1

		const aFormatted = formatCategoryName(a.name)
		const bFormatted = formatCategoryName(b.name)
		return aFormatted.localeCompare(bFormatted, locale, { numeric: true })
	})
}

export const capitalizeString = (name) => {
	return name ? name.charAt(0).toUpperCase() + name.slice(1) : name
}

export const formatWallet = (name) => {
	if (name === 'paypal') {
		return 'PayPal'
	}
	return capitalizeString(name)
}

export const formatProjectType = (name, short = false) => {
	if (short) {
		if (name === 'resourcepack') {
			return 'RPK'
		} else if (name === 'mod') {
			return 'MOD'
		} else if (name === 'modpack') {
			return 'MPK'
		} else if (name === 'shader') {
			return 'SHD'
		} else if (name === 'plugin') {
			return 'PLG'
		} else if (name === 'datapack') {
			return 'DPK'
		} else if (name === 'server') {
			return 'SRV'
		}
	}

	if (name === 'resourcepack') {
		return 'Resource Pack'
	} else if (name === 'datapack') {
		return 'Data Pack'
	} else if (name === 'modpack') {
		return 'Modpack'
	} else if (name === 'minecraft_java_server') {
		return 'Server'
	}

	return capitalizeString(name)
}

export const formatProjectStatus = (name) => {
	if (name === 'approved') {
		return 'Public'
	} else if (name === 'processing') {
		return 'Under review'
	}

	return capitalizeString(name)
}

export const formatVersions = (versionArray, gameVersions) => {
	const allVersions = gameVersions.slice().reverse()
	const allReleases = allVersions.filter((x) => x.version_type === 'release')

	const intervals = []
	let currentInterval = 0

	for (let i = 0; i < versionArray.length; i++) {
		const index = allVersions.findIndex((x) => x.version === versionArray[i])
		const releaseIndex = allReleases.findIndex((x) => x.version === versionArray[i])

		if (i === 0) {
			intervals.push([[versionArray[i], index, releaseIndex]])
		} else {
			const intervalBase = intervals[currentInterval]

			if (
				(index - intervalBase[intervalBase.length - 1][1] === 1 ||
					releaseIndex - intervalBase[intervalBase.length - 1][2] === 1) &&
				(allVersions[intervalBase[0][1]].version_type === 'release' ||
					allVersions[index].version_type !== 'release')
			) {
				intervalBase[1] = [versionArray[i], index, releaseIndex]
			} else {
				currentInterval += 1
				intervals[currentInterval] = [[versionArray[i], index, releaseIndex]]
			}
		}
	}

	const newIntervals = []
	for (let i = 0; i < intervals.length; i++) {
		const interval = intervals[i]

		if (interval.length === 2 && interval[0][2] !== -1 && interval[1][2] === -1) {
			let lastSnapshot = null
			for (let j = interval[1][1]; j > interval[0][1]; j--) {
				if (allVersions[j].version_type === 'release') {
					newIntervals.push([
						interval[0],
						[
							allVersions[j].version,
							j,
							allReleases.findIndex((x) => x.version === allVersions[j].version),
						],
					])

					if (lastSnapshot !== null && lastSnapshot !== j + 1) {
						newIntervals.push([[allVersions[lastSnapshot].version, lastSnapshot, -1], interval[1]])
					} else {
						newIntervals.push([interval[1]])
					}

					break
				} else {
					lastSnapshot = j
				}
			}
		} else {
			newIntervals.push(interval)
		}
	}

	const output = []

	for (const interval of newIntervals) {
		if (interval.length === 2) {
			output.push(`${interval[0][0]}–${interval[1][0]}`)
		} else {
			output.push(interval[0][0])
		}
	}

	return (output.length === 0 ? versionArray : output).join(', ')
}

export function cycleValue<T extends string>(value: T, values: T[]): T {
	const index = values.indexOf(value) + 1
	return values[index % values.length]
}

export const fileIsValid = (file, validationOptions, formatBytes) => {
	const { maxSize, alertOnInvalid } = validationOptions
	if (maxSize !== null && maxSize !== undefined && file.size > maxSize) {
		if (alertOnInvalid) {
			alert(`File ${file.name} is too big! Must be less than ${formatBytes(maxSize)}`)
		}
		return false
	}

	return true
}

export const acceptFileFromProjectType = (projectType) => {
	const commonTypes = '.sig,.asc,.gpg,application/pgp-signature,application/pgp-keys'
	switch (projectType) {
		case 'mod':
			return `.jar,.zip,.litemod,application/java-archive,application/x-java-archive,application/zip,${commonTypes}`
		case 'plugin':
			return `.jar,.zip,application/java-archive,application/x-java-archive,application/zip,${commonTypes}`
		case 'resourcepack':
			return `.zip,application/zip,${commonTypes}`
		case 'shader':
			return `.zip,application/zip,${commonTypes}`
		case 'datapack':
			return `.jar,.zip,.litemod,application/java-archive,application/x-java-archive,application/zip,${commonTypes}`
		case 'modpack':
			return `.mrpack,application/x-modrinth-modpack+zip,application/zip,${commonTypes}`
		default:
			// all of the above
			return `.jar,.zip,.litemod,.mrpack,application/java-archive,application/x-java-archive,application/zip,application/x-modrinth-modpack+zip,${commonTypes}`
	}
}

export const getArrayOrString = (x: string[] | string): string[] => {
	if (typeof x === 'string') {
		return [x]
	} else {
		return x
	}
}

export function getPingLevel(ping: number) {
	if (ping < 120) {
		return 5
	} else if (ping < 200) {
		return 4
	} else if (ping < 300) {
		return 3
	} else if (ping < 400) {
		return 2
	} else {
		return 1
	}
}

export function arrayBufferToBase64(buffer: Uint8Array | ArrayBuffer): string {
	const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer)
	return btoa(String.fromCharCode(...bytes))
}
export const DEFAULT_CREDIT_EMAIL_MESSAGE =
	"We're really sorry about the recent issues with your server."
