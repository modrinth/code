const projectTypeMessages = defineMessages({
  datapack: {
    id: 'project-type.datapack.singular',
    defaultMessage: 'Data Pack',
  },
  datapacks: {
    id: 'project-type.datapack.plural',
    defaultMessage: 'Data Packs',
  },
  mod: {
    id: 'project-type.mod.singular',
    defaultMessage: 'Mod',
  },
  mods: {
    id: 'project-type.mod.plural',
    defaultMessage: 'Mods',
  },
  modpack: {
    id: 'project-type.modpack.singular',
    defaultMessage: 'Modpack',
  },
  modpacks: {
    id: 'project-type.modpack.plural',
    defaultMessage: 'Modpacks',
  },
  plugin: {
    id: 'project-type.plugin.singular',
    defaultMessage: 'Plugin',
  },
  plugins: {
    id: 'project-type.plugin.plural',
    defaultMessage: 'Plugins',
  },
  resourcepack: {
    id: 'project-type.resourcepack.singular',
    defaultMessage: 'Resource Pack',
  },
  resourcepacks: {
    id: 'project-type.resourcepack.plural',
    defaultMessage: 'Resource Packs',
  },
  shader: {
    id: 'project-type.shader.singular',
    defaultMessage: 'Shader',
  },
  shaders: {
    id: 'project-type.shader.plural',
    defaultMessage: 'Shaders',
  },
  project: {
    id: 'project-type.project.singular',
    defaultMessage: 'Project',
  },
  projects: {
    id: 'project-type.project.plural',
    defaultMessage: 'Projects',
  },
  collection: {
    id: 'project-type.collection.singular',
    defaultMessage: 'Collection',
  },
  collections: {
    id: 'project-type.collection.plural',
    defaultMessage: 'Collections',
  },
})

type ExtractSingulars<K extends string> = K extends `${infer T}s` ? T : never

type ProjectType = ExtractSingulars<keyof typeof projectTypeMessages>

export function getProjectTypeMessage(type: ProjectType, plural = false) {
  return (
    projectTypeMessages[`${type}${plural ? 's' : ''}`] ??
    projectTypeMessages[`project${plural ? 's' : ''}`]
  )
}
