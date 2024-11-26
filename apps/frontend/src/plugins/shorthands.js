import { getProjectTypeForUrlShorthand } from "~/helpers/projects.js";

export default defineNuxtPlugin((nuxtApp) => {
  const tagStore = useTags();

  nuxtApp.provide("formatNumber", formatNumber);
  nuxtApp.provide("capitalizeString", capitalizeString);
  nuxtApp.provide("formatMoney", formatMoney);
  nuxtApp.provide("formatVersion", (versionsArray) => formatVersions(tagStore, versionsArray));
  nuxtApp.provide("orElse", (first, otherwise) => first ?? otherwise);
  nuxtApp.provide("external", () => {
    const cosmeticsStore = useCosmetics().value;

    return cosmeticsStore.externalLinksNewTab ? "_blank" : "";
  });
  nuxtApp.provide("formatBytes", formatBytes);
  nuxtApp.provide("formatWallet", formatWallet);
  nuxtApp.provide("formatProjectType", formatProjectType);
  nuxtApp.provide("formatCategory", formatCategory);
  nuxtApp.provide("formatCategoryHeader", formatCategoryHeader);

  /*
    Only use on the complete list of versions for a project, partial lists will generate
    the wrong version slugs
  */
  nuxtApp.provide("computeVersions", (versions, members) => {
    const visitedVersions = [];
    const returnVersions = [];

    const authorMembers = {};

    for (const version of versions.sort(
      (a, b) => nuxtApp.$dayjs(a.date_published) - nuxtApp.$dayjs(b.date_published),
    )) {
      if (visitedVersions.includes(version.version_number)) {
        visitedVersions.push(version.version_number);
        version.displayUrlEnding = version.id;
      } else {
        visitedVersions.push(version.version_number);
        version.displayUrlEnding = version.version_number;
      }
      version.primaryFile = version.files.find((file) => file.primary) ?? version.files[0];

      if (!version.primaryFile) {
        version.primaryFile = {
          hashes: {
            sha1: "",
            sha512: "",
          },
          url: "#",
          filename: "unknown",
          primary: false,
          size: 0,
          file_type: null,
        };
      }

      version.author = authorMembers[version.author_id];
      if (!version.author) {
        version.author = members.find((x) => x.user.id === version.author_id);
        authorMembers[version.author_id] = version.author;
      }

      returnVersions.push(version);
    }

    return returnVersions
      .reverse()
      .map((version, index) => {
        const nextVersion = returnVersions[index + 1];
        if (nextVersion && version.changelog && nextVersion.changelog === version.changelog) {
          return { duplicate: true, ...version };
        } else {
          return { duplicate: false, ...version };
        }
      })
      .sort((a, b) => nuxtApp.$dayjs(b.date_published) - nuxtApp.$dayjs(a.date_published));
  });
  nuxtApp.provide("getProjectTypeForDisplay", (type, categories) => {
    if (type === "mod") {
      const isPlugin = categories.some((category) => {
        return tagStore.value.loaderData.allPluginLoaders.includes(category);
      });
      const isMod = categories.some((category) => {
        return tagStore.value.loaderData.modLoaders.includes(category);
      });
      const isDataPack = categories.some((category) => {
        return tagStore.value.loaderData.dataPackLoaders.includes(category);
      });

      if (isMod && isPlugin && isDataPack) {
        return "模组, 插件, 和 数据包";
      } else if (isMod && isPlugin) {
        return "模组 和 插件";
      } else if (isMod && isDataPack) {
        return "模组 和 数据包";
      } else if (isPlugin && isDataPack) {
        return "插件 和 数据包";
      } else if (isDataPack) {
        return "数据包";
      } else if (isPlugin) {
        return "插件";
      }
    }

    return type;
  });
  nuxtApp.provide("getProjectTypeForUrl", (type, loaders, tags) =>
    getProjectTypeForUrlShorthand(type, loaders, tags),
  );
  nuxtApp.provide("cycleValue", cycleValue);
  nuxtApp.provide("sortedCategories", () => {
    return tagStore.value.categories.slice().sort((a, b) => {
      const headerCompare = a.header.localeCompare(b.header);
      if (headerCompare !== 0) {
        return headerCompare;
      }
      if (a.header === "resolutions" && b.header === "resolutions") {
        return a.name.replace(/\D/g, "") - b.name.replace(/\D/g, "");
      } else if (a.header === "performance impact" && b.header === "performance impact") {
        const x = ["potato", "low", "medium", "high", "screenshot"];

        return x.indexOf(a.name) - x.indexOf(b.name);
      }
      return 0;
    });
  });
  nuxtApp.provide("notify", (notif) => addNotification(notif));
});
export const formatNumber = (number, abbreviate = true) => {
  const x = +number;
  if (x >= 1000000 && abbreviate) {
    return (x / 1000000).toFixed(2).toString() + "M";
  } else if (x >= 10000 && abbreviate) {
    return (x / 1000).toFixed(1).toString() + "k";
  } else {
    return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  }
};

export const formatMoney = (number, abbreviate = false) => {
  number = Math.floor(number * 100) / 100;
  const x = +number;
  if (x >= 1000000 && abbreviate) {
    return "$" + (x / 1000000).toFixed(2).toString() + "M";
  } else if (x >= 10000 && abbreviate) {
    return "$" + (x / 1000).toFixed(2).toString() + "k";
  } else {
    return (
      "$" +
      x
        .toFixed(2)
        .toString()
        .replace(/\B(?=(\d{3})+(?!\d))/g, ",")
    );
  }
};

export const formatBytes = (bytes, decimals = 2) => {
  if (bytes === 0) {
    return "0 Bytes";
  }

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KiB", "MiB", "GiB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
};

export const capitalizeString = (name) => {
  return name ? name.charAt(0).toUpperCase() + name.slice(1) : name;
};

export const formatWallet = (name) => {
  if (name === "paypal") {
    return "PayPal";
  }
  return capitalizeString(name);
};

export const formatProjectType = (name) => {
  if (name === "resourcepack") {
    return "资源包";
  } else if (name === "datapack") {
    return "数据包";
  }

  return capitalizeString(name);
};

export const formatCategory = (name) => {
  if (name === "modloader") {
    return "ModLoader";
  } else if (name === "bungeecord") {
    return "BungeeCord";
  } else if (name === "liteloader") {
    return "LiteLoader";
  } else if (name === "neoforge") {
    return "NeoForge";
  } else if (name === "game-mechanics") {
    return "游戏机制";
  } else if (name === "worldgen") {
    return "世界生成";
  } else if (name === "core-shaders") {
    return "光影核心";
  } else if (name === "gui") {
    return "GUI";
  } else if (name === "8x-") {
    return "8x 或更低";
  } else if (name === "512x+") {
    return "512x 或更高";
  } else if (name === "kitchen-sink") {
    return "百宝箱模组包";
  } else if (name === "path-tracing") {
    return "路径追踪";
  } else if (name === "pbr") {
    return "物理渲染";
  } else if (name === "datapack") {
    return "数据包";
  } else if (name === "colored-lighting") {
    return "彩色光照";
  } else if (name === "optifine") {
    return "OptiFine高清修复";
  } else if (name === "mrpack") {
    return "整合包";
  } else if (name === "minecraft") {
    return "资源包";
  } else if (name === "vanilla") {
    return "原版光影";
  }

  return capitalizeString(name);
};

export const formatCategoryHeader = (name) => {
  return capitalizeString(name);
};

export const formatVersions = (tag, versionArray) => {
  const allVersions = tag.value.gameVersions.slice().reverse();
  const allReleases = allVersions.filter((x) => x.version_type === "release");

  const intervals = [];
  let currentInterval = 0;

  for (let i = 0; i < versionArray.length; i++) {
    const index = allVersions.findIndex((x) => x.version === versionArray[i]);
    const releaseIndex = allReleases.findIndex((x) => x.version === versionArray[i]);

    if (i === 0) {
      intervals.push([[versionArray[i], index, releaseIndex]]);
    } else {
      const intervalBase = intervals[currentInterval];

      if (
        (index - intervalBase[intervalBase.length - 1][1] === 1 ||
          releaseIndex - intervalBase[intervalBase.length - 1][2] === 1) &&
        (allVersions[intervalBase[0][1]].version_type === "release" ||
          allVersions[index].version_type !== "release")
      ) {
        intervalBase[1] = [versionArray[i], index, releaseIndex];
      } else {
        currentInterval += 1;
        intervals[currentInterval] = [[versionArray[i], index, releaseIndex]];
      }
    }
  }

  const newIntervals = [];
  for (let i = 0; i < intervals.length; i++) {
    const interval = intervals[i];

    if (interval.length === 2 && interval[0][2] !== -1 && interval[1][2] === -1) {
      let lastSnapshot = null;
      for (let j = interval[1][1]; j > interval[0][1]; j--) {
        if (allVersions[j].version_type === "release") {
          newIntervals.push([
            interval[0],
            [
              allVersions[j].version,
              j,
              allReleases.findIndex((x) => x.version === allVersions[j].version),
            ],
          ]);

          if (lastSnapshot !== null && lastSnapshot !== j + 1) {
            newIntervals.push([[allVersions[lastSnapshot].version, lastSnapshot, -1], interval[1]]);
          } else {
            newIntervals.push([interval[1]]);
          }

          break;
        } else {
          lastSnapshot = j;
        }
      }
    } else {
      newIntervals.push(interval);
    }
  }

  const output = [];

  for (const interval of newIntervals) {
    if (interval.length === 2) {
      output.push(`${interval[0][0]}–${interval[1][0]}`);
    } else {
      output.push(interval[0][0]);
    }
  }

  return (output.length === 0 ? versionArray : output).join(", ");
};

export const cycleValue = (value, values) => {
  const index = values.indexOf(value) + 1;
  return values[index % values.length];
};
