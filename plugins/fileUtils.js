import JSZip from 'jszip'
import TOML from '@iarna/toml'
import yaml from 'js-yaml'
import { formatBytes } from '~/plugins/shorthands'

/**
 * @param {File | Blob} file the file to validate
 * @param {{ maxSize: number, alertOnInvalid: boolean }} validationOptions the
 * constraints to validate the file against
 * @param validationOptions.maxSize the max file size in bytes
 * @param validationOptions.alertOnInvalid if an alert should pop up describing
 * each validation error
 * @returns `true` if the file is valid; `false` otherwise
 */
export const fileIsValid = (file, validationOptions) => {
  const { maxSize, alertOnInvalid } = validationOptions
  if (maxSize !== null && maxSize !== undefined && file.size > maxSize) {
    if (alertOnInvalid) {
      alert(
        `File ${file.name} is too big! Must be less than ${formatBytes(
          maxSize
        )}`
      )
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

export const inferVersionInfo = async function (
  rawFile,
  project,
  gameVersions
) {
  function versionType(number) {
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

  // TODO: This func does not handle accurate semver parsing. We should eventually
  function gameVersionRange(gameVersionString, gameVersions) {
    if (!gameVersionString) {
      return []
    }

    // Truncate characters after `-` & `+`
    const gameString = gameVersionString.replace(/-|\+.*$/g, '')

    let prefix = ''
    if (gameString.includes('~')) {
      // Include minor versions
      // ~1.2.3 -> 1.2
      prefix = gameString.replace('~', '').split('.').slice(0, 2).join('.')
    } else if (gameString.includes('>=')) {
      // Include minor versions
      // >=1.2.3 -> 1.2
      prefix = gameString.replace('>=', '').split('.').slice(0, 2).join('.')
    } else if (gameString.includes('^')) {
      // Include major versions
      // ^1.2.3 -> 1
      prefix = gameString.replace('^', '').split('.')[0]
    } else if (gameString.includes('x')) {
      // Include versions that match `x.x.x`
      // 1.2.x -> 1.2
      prefix = gameString.replace(/\.x$/, '')
    } else {
      // Include exact version
      // 1.2.3 -> 1.2.3
      prefix = gameString
    }

    const simplified = gameVersions
      .filter((it) => it.version_type === 'release')
      .map((it) => it.version)
    return simplified.filter((version) => version.startsWith(prefix))
  }

  const inferFunctions = {
    // Forge 1.13+
    'META-INF/mods.toml': async (file, zip) => {
      const metadata = TOML.parse(file)

      // TODO: Parse minecraft version ranges, handle if version is set to value from manifest
      if (metadata.mods && metadata.mods.length > 0) {
        let versionNum = metadata.mods[0].version

        // ${file.jarVersion} -> Implementation-Version from manifest
        const manifestFile = zip.file('META-INF/MANIFEST.MF')
        if (
          // eslint-disable-next-line no-template-curly-in-string
          metadata.mods[0].version.includes('${file.jarVersion}') &&
          manifestFile !== null
        ) {
          const manifestText = await manifestFile.async('text')
          const regex = /Implementation-Version: (.*)$/m
          const match = manifestText.match(regex)
          if (match) {
            // eslint-disable-next-line no-template-curly-in-string
            versionNum = versionNum.replace('${file.jarVersion}', match[1])
          }
        }

        return {
          name: `${project.title} ${versionNum}`,
          version_number: versionNum,
          version_type: versionType(versionNum),
          loaders: ['forge'],
        }
      } else {
        return {}
      }
    },
    // Old Forge
    'mcmod.info': (file) => {
      const metadata = JSON.parse(file)

      return {
        name: metadata.version ? `${project.title} ${metadata.version}` : '',
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        loaders: ['forge'],
        game_versions: gameVersions
          .filter(
            (x) =>
              x.version.startsWith(metadata.mcversion) &&
              x.version_type === 'release'
          )
          .map((x) => x.version),
      }
    },
    // Fabric
    'fabric.mod.json': (file) => {
      const metadata = JSON.parse(file)

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        loaders: ['fabric'],
        version_type: versionType(metadata.version),
        game_versions: metadata.depends
          ? gameVersionRange(metadata.depends.minecraft, gameVersions)
          : [],
      }
    },
    // Quilt
    'quilt.mod.json': (file) => {
      const metadata = JSON.parse(file)

      return {
        name: `${project.title} ${metadata.quilt_loader.version}`,
        version_number: metadata.quilt_loader.version,
        loaders: ['quilt'],
        version_type: versionType(metadata.quilt_loader.version),
        game_versions: metadata.quilt_loader.depends
          ? gameVersionRange(
              metadata.quilt_loader.depends.find((x) => x.id === 'minecraft')
                ? metadata.quilt_loader.depends.find(
                    (x) => x.id === 'minecraft'
                  ).versions
                : [],
              gameVersions
            )
          : [],
      }
    },
    // Bukkit + Other Forks
    'plugin.yml': (file) => {
      const metadata = yaml.load(file)

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        // We don't know which fork of Bukkit users are using
        loaders: [],
        game_versions: gameVersions
          .filter(
            (x) =>
              x.version.startsWith(metadata['api-version']) &&
              x.version_type === 'release'
          )
          .map((x) => x.version),
      }
    },
    // Bungeecord + Waterfall
    'bungee.yml': (file) => {
      const metadata = yaml.load(file)

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        loaders: ['bungeecord'],
      }
    },
    // Modpacks
    'modrinth.index.json': (file) => {
      const metadata = JSON.parse(file)

      const loaders = []
      if ('forge' in metadata.dependencies) loaders.push('forge')
      if ('fabric-loader' in metadata.dependencies) loaders.push('fabric')
      if ('quilt-loader' in metadata.dependencies) loaders.push('quilt')

      return {
        name: `${project.title} ${metadata.versionId}`,
        version_number: metadata.versionId,
        version_type: versionType(metadata.versionId),
        loaders,
        game_versions: gameVersions
          .filter((x) => x.version === metadata.dependencies.minecraft)
          .map((x) => x.version),
      }
    },
    // Resource Packs + Data Packs
    'pack.mcmeta': (file) => {
      const metadata = JSON.parse(file)

      function getRange(versionA, versionB) {
        const startingIndex = gameVersions.findIndex(
          (x) => x.version === versionA
        )
        const endingIndex = gameVersions.findIndex(
          (x) => x.version === versionB
        )

        const final = []
        const filterOnlyRelease =
          gameVersions[startingIndex].version_type === 'release'

        for (let i = startingIndex; i >= endingIndex; i--) {
          if (
            gameVersions[i].version_type === 'release' ||
            !filterOnlyRelease
          ) {
            final.push(gameVersions[i].version)
          }
        }

        return final
      }

      const loaders = []
      let newGameVersions = []

      if (project.actualProjectType === 'mod') {
        loaders.push('datapack')

        switch (metadata.pack.pack_format) {
          case 4:
            newGameVersions = getRange('1.13', '1.14.4')
            break
          case 5:
            newGameVersions = getRange('1.15', '1.16.1')
            break
          case 6:
            newGameVersions = getRange('1.16.2', '1.16.5')
            break
          case 7:
            newGameVersions = getRange('1.17', '1.17.1')
            break
          case 8:
            newGameVersions = getRange('1.18', '1.18.1')
            break
          case 9:
            newGameVersions.push('1.18.2')
            break
          case 10:
            newGameVersions = getRange('1.19', '1.19.3')
            break
          default:
        }
      }

      if (project.actualProjectType === 'resourcepack') {
        loaders.push('minecraft')

        switch (metadata.pack.pack_format) {
          case 1:
            newGameVersions = getRange('1.6.1', '1.8.9')
            break
          case 2:
            newGameVersions = getRange('1.9', '1.10.2')
            break
          case 3:
            newGameVersions = getRange('1.11', '1.12.2')
            break
          case 4:
            newGameVersions = getRange('1.13', '1.14.4')
            break
          case 5:
            newGameVersions = getRange('1.15', '1.16.1')
            break
          case 6:
            newGameVersions = getRange('1.16.2', '1.16.5')
            break
          case 7:
            newGameVersions = getRange('1.17', '1.17.1')
            break
          case 8:
            newGameVersions = getRange('1.18', '1.18.2')
            break
          case 9:
            newGameVersions = getRange('1.19', '1.19.2')
            break
          case 11:
            newGameVersions = getRange('22w42a', '22w44a')
            break
          case 12:
            newGameVersions.push('1.19.3')
            break
          default:
        }
      }

      return {
        loaders,
        game_versions: newGameVersions,
      }
    },
  }

  const zipReader = new JSZip()

  const zip = await zipReader.loadAsync(rawFile)

  for (const fileName in inferFunctions) {
    const file = zip.file(fileName)

    if (file !== null) {
      const text = await file.async('text')
      return inferFunctions[fileName](text, zip)
    }
  }
}

export const createDataPackVersion = async function (
  project,
  version,
  primaryFile,
  members,
  allGameVersions,
  loaders
) {
  // force version to start with number, as required by FML
  const newVersionNumber = version.version_number.match(/^\d/)
    ? version.version_number
    : `1-${version.version_number}`

  const targetStartingDigitsRegex = /^(\d+)(\D+)$/g
  const newSlug = `${project.slug
    .replace('-', '_')
    .replace(/\W/g, '')
    .replace(targetStartingDigitsRegex, '$2')
    .replace(/^(\d+)$/g, project.id.replace(targetStartingDigitsRegex, '$2'))
    .substring(0, 63)}_mr`

  const iconPath = `${project.slug}_pack.png`

  const fabricModJson = {
    schemaVersion: 1,
    id: newSlug,
    version: newVersionNumber,
    name: project.title,
    description: project.description,
    authors: members.map((x) => x.name),
    contact: {
      homepage: `${process.env.domain}/${project.project_type}/${
        project.slug ?? project.id
      }`,
    },
    license: project.license.id,
    icon: iconPath,
    environment: '*',
    depends: {
      'fabric-resource-loader-v0': '*',
    },
  }

  const quiltModJson = {
    schema_version: 1,
    quilt_loader: {
      group: 'com.modrinth',
      id: newSlug,
      version: newVersionNumber,
      metadata: {
        name: project.title,
        description: project.description,
        contributors: members.reduce(
          (acc, x) => ({
            ...acc,
            [x.name]: x.role,
          }),
          {}
        ),
        contact: {
          homepage: `${process.env.domain}/${project.project_type}/${
            project.slug ?? project.id
          }`,
        },
        icon: iconPath,
      },
      intermediate_mappings: 'net.fabricmc:intermediary',
      depends: [
        {
          id: 'quilt_resource_loader',
          versions: '*',
          unless: 'fabric-resource-loader-v0',
        },
      ],
    },
  }

  const cutoffIndex = allGameVersions.findIndex((x) => x.version === '1.18.2')

  let maximumIndex = Number.MIN_VALUE
  for (const val of version.game_versions) {
    const index = allGameVersions.findIndex((x) => x.version === val)
    if (index > maximumIndex) {
      maximumIndex = index
    }
  }

  const newForge = maximumIndex < cutoffIndex

  const forgeModsToml = {
    modLoader: newForge ? 'lowcodefml' : 'javafml',
    loaderVersion: newForge ? '[40,)' : '[25,)',
    license: project.license.id,
    showAsResourcePack: false,
    mods: [
      {
        modId: newSlug,
        version: newVersionNumber,
        displayName: project.title,
        description: project.description,
        logoFile: iconPath,
        updateJSONURL: `${process.env.authURLBase.replace(
          '/v2/',
          ''
        )}/updates/${project.id}/forge_updates.json`,
        credits: 'Generated by Modrinth',
        authors: members.map((x) => x.name).join(', '),
        displayURL: `${process.env.domain}/${project.project_type}/${
          project.slug ?? project.id
        }`,
      },
    ],
  }

  if (project.source_url) {
    quiltModJson.quilt_loader.metadata.contact.sources = project.source_url
    fabricModJson.contact.sources = project.source_url
  }

  if (project.issues_url) {
    quiltModJson.quilt_loader.metadata.contact.issues = project.issues_url
    fabricModJson.contact.issues = project.issues_url
    forgeModsToml.issueTrackerURL = project.issues_url
  }

  const primaryFileData = await (await fetch(primaryFile.url)).blob()

  const primaryZipReader = new JSZip()
  await primaryZipReader.loadAsync(primaryFileData)

  if (loaders.includes('fabric'))
    primaryZipReader.file('fabric.mod.json', JSON.stringify(fabricModJson))
  if (loaders.includes('quilt'))
    primaryZipReader.file('quilt.mod.json', JSON.stringify(quiltModJson))
  if (loaders.includes('forge'))
    primaryZipReader.file('META-INF/mods.toml', TOML.stringify(forgeModsToml))

  if (!newForge && loaders.includes('forge')) {
    const classFile = new Uint8Array(
      await (
        await fetch(
          'https://cdn.modrinth.com/wrapper/ModrinthWrapperRestiched.class'
        )
      ).arrayBuffer()
    )

    let binary = ''
    for (let i = 0; i < classFile.byteLength; i++) {
      binary += String.fromCharCode(classFile[i])
    }

    let sanitizedId = project.id

    if (project.id.match(/^(\d+)/g)) {
      sanitizedId = '_' + sanitizedId
    }

    sanitizedId = sanitizedId.substring(0, 8)

    binary = binary
      .replace(
        String.fromCharCode(32) + 'needs1to1be1changed1modrinth1mod',
        String.fromCharCode(newSlug.length) + newSlug
      )
      .replace('/wrappera/', `/${sanitizedId}/`)

    const newArr = []
    for (let i = 0; i < binary.length; i++) {
      newArr.push(binary.charCodeAt(i))
    }

    primaryZipReader.file(
      `com/modrinth/${sanitizedId}/ModrinthWrapper.class`,
      new Uint8Array(newArr)
    )
  }

  const resourcePack = version.files.find(
    (x) => x.file_type === 'required-resource-pack'
  )

  const resourcePackData = resourcePack
    ? await (await fetch(resourcePack.url)).blob()
    : null

  if (resourcePackData) {
    const resourcePackReader = new JSZip()
    await resourcePackReader.loadAsync(resourcePackData)

    for (const [path, file] of Object.entries(resourcePackReader.files)) {
      if (!primaryZipReader.file(path) && !path.includes('.mcassetsroot')) {
        primaryZipReader.file(path, await file.async('uint8array'))
      }
    }
  }

  if (primaryZipReader.file('pack.png')) {
    primaryZipReader.file(
      iconPath,
      await primaryZipReader.file('pack.png').async('uint8array')
    )
  }

  return await primaryZipReader.generateAsync({
    type: 'blob',
    mimeType: 'application/java-archive',
  })
}
