import { formatBytes } from "~/plugins/shorthands.js";

export const fileIsValid = (file, validationOptions) => {
  const { maxSize, alertOnInvalid } = validationOptions;
  if (maxSize !== null && maxSize !== undefined && file.size > maxSize) {
    if (alertOnInvalid) {
      alert(`File ${file.name} is too big! Must be less than ${formatBytes(maxSize)}`);
    }
    return false;
  }

  return true;
};

export const acceptFileFromProjectType = (projectType) => {
  switch (projectType) {
    case "mod":
      return ".jar,.zip,.litemod,application/java-archive,application/x-java-archive,application/zip";
    case "plugin":
      return ".jar,.zip,application/java-archive,application/x-java-archive,application/zip";
    case "resourcepack":
      return ".zip,application/zip";
    case "shader":
      return ".zip,application/zip";
    case "datapack":
      return ".zip,application/zip";
    case "modpack":
      return ".mrpack,application/x-modrinth-modpack+zip,application/zip";
    default:
      return "*";
  }
};
