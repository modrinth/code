// noinspection JSUnusedGlobalSymbols
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
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

export const sortedCategories = (tags) => {
  return tags.categories.slice().sort((a, b) => {
    const headerCompare = a.header.localeCompare(b.header)
    if (headerCompare !== 0) {
      return headerCompare
    }
    if (a.header === 'resolutions' && b.header === 'resolutions') {
      return a.name.replace(/\D/g, '') - b.name.replace(/\D/g, '')
    } else if (a.header === 'performance impact' && b.header === 'performance impact') {
      const x = ['potato', 'low', 'medium', 'high', 'screenshot']

      return x.indexOf(a.name) - x.indexOf(b.name)
    }
    return 0
  })
}

export const formatNumber = (number, abbreviate = true) => {
  const x = Number(number)
  if (x >= 1000000 && abbreviate) {
    return `${(x / 1000000).toFixed(2).toString()}M`
  } else if (x >= 10000 && abbreviate) {
    return `${(x / 1000).toFixed(1).toString()}k`
  }
  return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
}

export function formatDate(
  date: dayjs.Dayjs,
  options: Intl.DateTimeFormatOptions = {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  },
): string {
  return date.toDate().toLocaleDateString(undefined, options)
}

export function formatMoney(number, abbreviate = false) {
  const x = Number(number)
  if (x >= 1000000 && abbreviate) {
    return `$${(x / 1000000).toFixed(2).toString()}M`
  } else if (x >= 10000 && abbreviate) {
    return `$${(x / 1000).toFixed(2).toString()}k`
  }
  return `$${x
    .toFixed(2)
    .toString()
    .replace(/\B(?=(\d{3})+(?!\d))/g, ',')}`
}

export const formatBytes = (bytes, decimals = 2) => {
  if (bytes === 0) return '0 Bytes'

  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KiB', 'MiB', 'GiB']

  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`
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

export const formatProjectType = (name) => {
  if (name === 'resourcepack') {
    return 'Resource Pack'
  } else if (name === 'datapack') {
    return 'Data Pack'
  }

  return capitalizeString(name)
}

export const formatCategory = (name) => {
  if (name === 'modloader') {
    return "Risugami's ModLoader"
  } else if (name === 'bungeecord') {
    return 'BungeeCord'
  } else if (name === 'liteloader') {
    return 'LiteLoader'
  } else if (name === 'neoforge') {
    return 'NeoForge'
  } else if (name === 'game-mechanics') {
    return 'Game Mechanics'
  } else if (name === 'worldgen') {
    return 'World Generation'
  } else if (name === 'core-shaders') {
    return 'Core Shaders'
  } else if (name === 'gui') {
    return 'GUI'
  } else if (name === '8x-') {
    return '8x or lower'
  } else if (name === '512x+') {
    return '512x or higher'
  } else if (name === 'kitchen-sink') {
    return 'Kitchen Sink'
  } else if (name === 'path-tracing') {
    return 'Path Tracing'
  } else if (name === 'pbr') {
    return 'PBR'
  } else if (name === 'datapack') {
    return 'Data Pack'
  } else if (name === 'colored-lighting') {
    return 'Colored Lighting'
  } else if (name === 'optifine') {
    return 'OptiFine'
  } else if (name === 'bta-babric') {
    return 'BTA (Babric)'
  } else if (name === 'legacy-fabric') {
    return 'Legacy Fabric'
  } else if (name === 'java-agent') {
    return 'Java Agent'
  } else if (name === 'nilloader') {
    return 'NilLoader'
  } else if (name === 'mrpack') {
    return 'Modpack'
  } else if (name === 'minecraft') {
    return 'Resource Pack'
  } else if (name === 'vanilla') {
    return 'Vanilla Shader'
  }
  return capitalizeString(name)
}

export const formatCategoryHeader = (name) => {
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

export function cycleValue(value, values) {
  const index = values.indexOf(value) + 1
  return values[index % values.length]
}

export const fileIsValid = (file, validationOptions) => {
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
  switch (projectType) {
    case 'mod':
      return '.jar,.zip,.litemod,application/java-archive,application/x-java-archive,application/zip'
    case 'plugin':
      return '.jar,.zip,application/java-archive,application/x-java-archive,application/zip'
    case 'resourcepack':
      return '.zip,application/zip'
    case 'shader':
      return '.zip,application/zip'
    case 'datapack':
      return '.zip,application/zip'
    case 'modpack':
      return '.mrpack,application/x-modrinth-modpack+zip,application/zip'
    default:
      return '*'
  }
}

// Sorts alphabetically, but correctly identifies 8x, 128x, 256x, etc
// identifier[0], then if it ties, identifier[1], etc
export const sortByNameOrNumber = (sortable, identifiers) => {
  sortable.sort((a, b) => {
    for (const identifier of identifiers) {
      const aNum = parseFloat(a[identifier])
      const bNum = parseFloat(b[identifier])
      if (isNaN(aNum) && isNaN(bNum)) {
        // Both are strings, sort alphabetically
        const stringComp = a[identifier].localeCompare(b[identifier])
        if (stringComp != 0) return stringComp
      } else if (!isNaN(aNum) && !isNaN(bNum)) {
        // Both are numbers, sort numerically
        const numComp = aNum - bNum
        if (numComp != 0) return numComp
      } else {
        // One is a number and one is a string, numbers go first
        const numStringComp = isNaN(aNum) ? 1 : -1
        if (numStringComp != 0) return numStringComp
      }
    }
    return 0
  })
  return sortable
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
