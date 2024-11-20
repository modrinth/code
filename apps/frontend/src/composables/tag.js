import tags from "~/generated/state.json";

export const useTags = () =>
  useState("tags", () => ({
    categories: tags.categories,
    loaders: tags.loaders,
    gameVersions: tags.gameVersions,
    donationPlatforms: tags.donationPlatforms,
    reportTypes: tags.reportTypes,
    projectTypes: [
      {
        actual: "mod",
        id: "mod",
        display: "模组",
      },
      {
        actual: "mod",
        id: "plugin",
        display: "插件",
      },
      {
        actual: "mod",
        id: "datapack",
        display: "数据包",
      },
      {
        actual: "shader",
        id: "shader",
        display: "光影包",
      },
      {
        actual: "resourcepack",
        id: "resourcepack",
        display: "资源包",
      },
      {
        actual: "modpack",
        id: "modpack",
        display: "模组整合包",
      },
    ],
    loaderData: {
      pluginLoaders: ["bukkit", "spigot", "paper", "purpur", "sponge", "folia"],
      pluginPlatformLoaders: ["bungeecord", "waterfall", "velocity"],
      allPluginLoaders: [
        "bukkit",
        "spigot",
        "paper",
        "purpur",
        "sponge",
        "bungeecord",
        "waterfall",
        "velocity",
        "folia",
      ],
      dataPackLoaders: ["datapack"],
      modLoaders: ["forge", "fabric", "quilt", "liteloader", "modloader", "rift", "neoforge"],
      hiddenModLoaders: ["liteloader", "modloader", "rift"],
    },
    projectViewModes: ["list", "grid", "gallery"],
    approvedStatuses: ["approved", "archived", "unlisted", "private"],
    rejectedStatuses: ["rejected", "withheld"],
    staffRoles: ["moderator", "admin"],
  }));
