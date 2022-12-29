import tags from '~/generated/state.json'

export const state = () => ({
  categories: tags.categories,
  loaders: tags.loaders,
  gameVersions: tags.gameVersions,
  donationPlatforms: tags.donationPlatforms,
  reportTypes: tags.reportTypes,
  projectTypes: [
    {
      actual: 'mod',
      id: 'mod',
      display: 'mod',
    },
    {
      actual: 'mod',
      id: 'plugin',
      display: 'plugin',
    },
    {
      actual: 'mod',
      id: 'datapack',
      display: 'datapack',
    },
    {
      actual: 'resourcepack',
      id: 'resourcepack',
      display: 'resource pack',
    },
    {
      actual: 'modpack',
      id: 'modpack',
      display: 'modpack',
    },
    {
      actual: 'shader',
      id: 'shader',
      display: 'shader',
    },
  ],
  loaderData: {
    pluginLoaders: ['bukkit', 'spigot', 'paper', 'purpur', 'sponge'],
    pluginPlatformLoaders: ['bungeecord', 'waterfall', 'velocity'],
    allPluginLoaders: [
      'bukkit',
      'spigot',
      'paper',
      'purpur',
      'sponge',
      'bungeecord',
      'waterfall',
      'velocity',
    ],
    dataPackLoaders: ['datapack'],
    modLoaders: ['forge', 'fabric', 'quilt', 'liteloader', 'modloader', 'rift'],
  },
  projectViewModes: ['list', 'grid', 'gallery'],
})
