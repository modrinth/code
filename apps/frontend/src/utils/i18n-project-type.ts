const projectTypeMessages = defineMessages({
  datapack: {
    id: "project-type.datapack.singular",
    defaultMessage: "数据包",
  },
  datapacks: {
    id: "project-type.datapack.plural",
    defaultMessage: "数据包",
  },
  mod: {
    id: "project-type.mod.singular",
    defaultMessage: "模组",
  },
  mods: {
    id: "project-type.mod.plural",
    defaultMessage: "模组",
  },
  modpack: {
    id: "project-type.modpack.singular",
    defaultMessage: "模组整合包",
  },
  modpacks: {
    id: "project-type.modpack.plural",
    defaultMessage: "模组整合包",
  },
  plugin: {
    id: "project-type.plugin.singular",
    defaultMessage: "服务端插件",
  },
  plugins: {
    id: "project-type.plugin.plural",
    defaultMessage: "服务端插件",
  },
  resourcepack: {
    id: "project-type.resourcepack.singular",
    defaultMessage: "资源包",
  },
  resourcepacks: {
    id: "project-type.resourcepack.plural",
    defaultMessage: "资源包",
  },
  shader: {
    id: "project-type.shader.singular",
    defaultMessage: "光影",
  },
  shaders: {
    id: "project-type.shader.plural",
    defaultMessage: "光影",
  },
  project: {
    id: "project-type.project.singular",
    defaultMessage: "资源",
  },
  projects: {
    id: "project-type.project.plural",
    defaultMessage: "资源",
  },
  collection: {
    id: "project-type.collection.singular",
    defaultMessage: "收藏",
  },
  collections: {
    id: "project-type.collection.plural",
    defaultMessage: "收藏",
  },
});

type ExtractSingulars<K extends string> = K extends `${infer T}s` ? T : never;

type ProjectType = ExtractSingulars<keyof typeof projectTypeMessages>;

export function getProjectTypeMessage(type: ProjectType, plural = false) {
  return (
    projectTypeMessages[`${type}${plural ? "s" : ""}`] ??
    projectTypeMessages[`project${plural ? "s" : ""}`]
  );
}
