import TOML from '@ltd/j-toml'
import JSZip from 'jszip'
import yaml from 'js-yaml'

export const inferVersionInfo = async function (rawFile, project, gameVersions) {
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
          .filter((x) => x.version.startsWith(metadata.mcversion) && x.version_type === 'release')
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
                ? metadata.quilt_loader.depends.find((x) => x.id === 'minecraft').versions
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
            (x) => x.version.startsWith(metadata['api-version']) && x.version_type === 'release'
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
      if ('forge' in metadata.dependencies) {
        loaders.push('forge')
      }
      if ('fabric-loader' in metadata.dependencies) {
        loaders.push('fabric')
      }
      if ('quilt-loader' in metadata.dependencies) {
        loaders.push('quilt')
      }

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
        const startingIndex = gameVersions.findIndex((x) => x.version === versionA)
        const endingIndex = gameVersions.findIndex((x) => x.version === versionB)

        const final = []
        const filterOnlyRelease = gameVersions[startingIndex].version_type === 'release'

        for (let i = startingIndex; i >= endingIndex; i--) {
          if (gameVersions[i].version_type === 'release' || !filterOnlyRelease) {
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
