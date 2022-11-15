export default (ctx, inject) => {
  inject('user', ctx.store.state.user)
  inject('tag', ctx.store.state.tag)
  inject('auth', ctx.store.state.auth)
  inject('defaultHeaders', () => {
    const obj = { headers: {} }

    if (process.server && process.env.RATE_LIMIT_IGNORE_KEY) {
      obj.headers['x-ratelimit-key'] = process.env.RATE_LIMIT_IGNORE_KEY || ''
    }

    if (ctx.store.state.auth.user) {
      obj.headers.Authorization = ctx.store.state.auth.token
    }

    return obj
  })
  inject('formatNumber', formatNumber)
  inject('formatMoney', (number) => '$' + formatNumber(number.toFixed(2)))
  inject('formatVersion', (versionsArray) =>
    formatVersions(versionsArray, ctx.store)
  )
  inject('orElse', (first, otherwise) => first ?? otherwise)
  inject('formatBytes', formatBytes)
  inject('formatWallet', formatWallet)
  inject('formatProjectType', formatProjectType)
  inject('formatCategory', formatCategory)
  inject('formatCategoryHeader', formatCategoryHeader)
  inject('computeVersions', (versions) => {
    const versionsMap = {}

    for (const version of versions.sort(
      (a, b) => ctx.$dayjs(a.date_published) - ctx.$dayjs(b.date_published)
    )) {
      if (versionsMap[version.version_number]) {
        versionsMap[version.version_number].push(version)
      } else {
        versionsMap[version.version_number] = [version]
      }
    }

    const returnVersions = []

    for (const id in versionsMap) {
      const versions = versionsMap[id]

      if (versions.length === 1) {
        versions[0].displayUrlEnding = versions[0].version_number

        returnVersions.push(versions[0])
      } else {
        const reservedNames = {}

        const seenLoaders = {}
        const duplicateLoaderIndexes = []

        for (let i = 0; i < versions.length; i++) {
          const version = versions[i]
          const value = version.loaders.join('+')

          if (seenLoaders[value]) {
            duplicateLoaderIndexes.push(i)
          } else {
            if (i !== 0) {
              version.displayUrlEnding = `${version.version_number}-${value}`
            } else {
              version.displayUrlEnding = version.version_number
            }

            reservedNames[version.displayUrlEnding] = true

            version.displayName = version.loaders
              .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
              .join(', ')

            returnVersions.push(version)

            seenLoaders[value] = true
          }
        }

        const seenGameVersions = {}
        const duplicateGameVersionIndexes = []

        for (const i of duplicateLoaderIndexes) {
          const version = versions[i]
          const value = version.game_versions.join('+')

          if (seenGameVersions[value]) {
            duplicateGameVersionIndexes.push(i)
          } else {
            if (i !== 0) {
              let setDisplayUrl = false

              for (const gameVersion in version.game_versions) {
                const displayUrlEnding = `${version.version_number}-${gameVersion}`

                if (!reservedNames[version.version_number]) {
                  version.displayUrlEnding = displayUrlEnding
                  reservedNames[displayUrlEnding] = true
                  setDisplayUrl = true

                  break
                }
              }

              if (!setDisplayUrl) {
                version.displayUrlEnding = `${version.version_number}-${value}`
              }
            } else if (!reservedNames[version.version_number]) {
              version.displayUrlEnding = version.version_number
              reservedNames[version.version_number] = true
            }

            version.displayName = formatVersions(
              version.game_versions,
              ctx.store
            )

            returnVersions.push(version)

            seenGameVersions[value] = true
          }
        }

        for (const i in duplicateGameVersionIndexes) {
          const version = versions[i]

          version.displayUrlEnding = version.id
          version.displayName = version.id

          returnVersions.push(version)
        }
      }
    }

    return returnVersions.sort(
      (a, b) => ctx.$dayjs(b.date_published) - ctx.$dayjs(a.date_published)
    )
  })
  inject('getProjectTypeForDisplay', (type, categories) => {
    if (type === 'mod') {
      const isPlugin = categories.some((category) => {
        return ctx.store.state.tag.loaderData.allPluginLoaders.includes(
          category
        )
      })
      const isMod = categories.some((category) => {
        return ctx.store.state.tag.loaderData.modLoaders.includes(category)
      })
      return isPlugin && isMod ? 'mod and plugin' : isPlugin ? 'plugin' : 'mod'
    } else {
      return formatProjectType(type)
    }
  })
  inject('getProjectTypeForUrl', (type, categories) => {
    if (type === 'mod') {
      const isPlugin = categories.some((category) => {
        return ctx.store.state.tag.loaderData.allPluginLoaders.includes(
          category
        )
      })

      const isMod = categories.some((category) => {
        return ctx.store.state.tag.loaderData.modLoaders.includes(category)
      })

      return isPlugin && isMod ? 'mod' : isPlugin ? 'plugin' : 'mod'
    } else {
      return type
    }
  })
}

export const formatNumber = (number) => {
  const x = +number
  if (x >= 1000000) {
    return (x / 1000000).toFixed(2).toString() + 'M'
  } else if (x >= 10000) {
    return (x / 1000).toFixed(1).toString() + 'K'
  } else {
    return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  }
}

export const formatBytes = (bytes, decimals = 2) => {
  if (bytes === 0) return '0 Bytes'

  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KiB', 'MiB', 'GiB']

  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

export const formatWallet = (name) => {
  if (name === 'paypal') {
    return 'PayPal'
  }
  return name.charAt(0).toUpperCase() + name.slice(1)
}

export const formatProjectType = (name) => {
  if (name === 'resourcepack') {
    return 'Resource Pack'
  }
  return name.charAt(0).toUpperCase() + name.slice(1)
}

export const formatCategory = (name) => {
  if (name === 'modloader') {
    return "Risugami's ModLoader"
  } else if (name === 'bungeecord') {
    return 'BungeeCord'
  } else if (name === 'liteloader') {
    return 'LiteLoader'
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
  }
  return name.charAt(0).toUpperCase() + name.slice(1)
}

export const formatCategoryHeader = (name) => {
  return name.charAt(0).toUpperCase() + name.slice(1)
}

export const formatVersions = (versionArray, store) => {
  const allVersions = store.state.tag.gameVersions.slice().reverse()
  const allReleases = allVersions.filter((x) => x.version_type === 'release')

  const intervals = []
  let currentInterval = 0

  for (let i = 0; i < versionArray.length; i++) {
    const index = allVersions.findIndex((x) => x.version === versionArray[i])
    const releaseIndex = allReleases.findIndex(
      (x) => x.version === versionArray[i]
    )

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

    if (
      interval.length === 2 &&
      interval[0][2] !== -1 &&
      interval[1][2] === -1
    ) {
      let lastSnapshot = null
      for (let j = interval[1][1]; j > interval[0][1]; j--) {
        if (allVersions[j].version_type === 'release') {
          newIntervals.push([
            interval[0],
            [
              allVersions[j].version,
              j,
              allReleases.findIndex(
                (x) => x.version === allVersions[j].version
              ),
            ],
          ])

          if (lastSnapshot !== null && lastSnapshot !== j + 1) {
            newIntervals.push([
              [allVersions[lastSnapshot].version, lastSnapshot, -1],
              interval[1],
            ])
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
      output.push(`${interval[0][0]}—${interval[1][0]}`)
    } else {
      output.push(interval[0][0])
    }
  }

  return output.join(', ')
}
