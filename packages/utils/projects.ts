// noinspection JSUnusedGlobalSymbols

export const isApproved = (project) => {
  return project && APPROVED_PROJECT_STATUSES.includes(project.status)
}

export const isListed = (project) => {
  return project && LISTED_PROJECT_STATUSES.includes(project.status)
}

export const isUnlisted = (project) => {
  return project && UNLISTED_PROJECT_STATUSES.includes(project.status)
}

export const isPrivate = (project) => {
  return project && PRIVATE_PROJECT_STATUSES.includes(project.status)
}

export const isRejected = (project) => {
  return project && REJECTED_PROJECT_STATUSES.includes(project.status)
}

export const isUnderReview = (project) => {
  return project && UNDER_REVIEW_PROJECT_STATUSES.includes(project.status)
}

export const isDraft = (project) => {
  return project && DRAFT_PROJECT_STATUSES.includes(project.status)
}

export const APPROVED_PROJECT_STATUSES = ['approved', 'archived', 'unlisted', 'private']
export const LISTED_PROJECT_STATUSES = ['approved', 'archived']
export const UNLISTED_PROJECT_STATUSES = ['unlisted', 'withheld']
export const PRIVATE_PROJECT_STATUSES = ['private', 'rejected', 'processing']
export const REJECTED_PROJECT_STATUSES = ['rejected', 'withheld']
export const UNDER_REVIEW_PROJECT_STATUSES = ['processing']
export const DRAFT_PROJECT_STATUSES = ['draft']

export type GameVersionTag = {
  version: string
  version_type: string
  date: string
  major: boolean
}

export type DisplayProjectType =
  | 'mod'
  | 'plugin'
  | 'datapack'
  | 'resourcepack'
  | 'modpack'
  | 'shader'

export type PlatformTag = {
  icon: string
  name: string
  supported_project_types: DisplayProjectType[]
}

export function getVersionsToDisplay(project, allGameVersions: GameVersionTag[]) {
  return formatVersionsForDisplay(project.game_versions.slice(), allGameVersions)
}

export function formatVersionsForDisplay(
  gameVersions: string[],
  allGameVersions: GameVersionTag[],
) {
  const inputVersions = gameVersions.slice()
  const allVersions = allGameVersions.slice()

  const allSnapshots = allVersions.filter((version) => version.version_type === 'snapshot')
  const allReleases = allVersions.filter((version) => version.version_type === 'release')
  const allLegacy = allVersions.filter(
    (version) => version.version_type !== 'snapshot' && version.version_type !== 'release',
  )

  {
    const indices = allVersions.reduce((map, gameVersion, index) => {
      map[gameVersion.version] = index
      return map
    }, {})
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

function groupConsecutiveIndices(versions: string[], referenceList: GameVersionTag[]) {
  if (!versions || versions.length === 0) {
    return []
  }

  const referenceMap = new Map()
  referenceList.forEach((item, index) => {
    referenceMap.set(item.version, index)
  })

  const sortedList: string[] = versions
    .slice()
    .sort((a, b) => referenceMap.get(a) - referenceMap.get(b))

  const ranges: string[] = []
  let start = sortedList[0]
  let previous = sortedList[0]

  for (let i = 1; i < sortedList.length; i++) {
    const current = sortedList[i]
    if (referenceMap.get(current) !== referenceMap.get(previous) + 1) {
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
