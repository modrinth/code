/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'
import { create } from './profile'

/*
  API for importing instances from other launchers
  launcherType can be one of the following:
  - MultiMC
  - GDLauncher
  - ATLauncher
  - Curseforge
  - PrismLauncher
  - Unknown (shouldn't be used, but is used internally if the launcher type isn't recognized)

  For each launcher type, we can get a guess of the default path for the launcher, and a list of importable instances
  For most launchers, this will be the application's data directory, with two exceptions:
  - MultiMC: this goes to the app directory (wherever the app is)
  - Curseforge: this goes to the 'minecraft' subdirectory of the data directory, as Curseforge has multiple games

*/

/// Gets a list of importable instances from a launcher type and base path
/// eg: get_importable_instances("MultiMC", "C:/MultiMC")
/// returns ["Instance 1", "Instance 2"]
export async function get_importable_instances(launcherType, basePath) {
  return await invoke('plugin:import|get_importable_instances', { launcherType, basePath })
}

/// Import an instance from a launcher type and base path
/// eg: import_instance("profile-name-to-go-to", "MultiMC", "C:/MultiMC", "Instance 1")
export async function import_instance(launcherType, basePath, instanceFolder) {
  // create a basic, empty instance (most properties will be filled in by the import process)
  // We do NOT watch the fs for changes to avoid duplicate events during installation
  // fs watching will be enabled once the instance is imported
  const profilePath = await create(instanceFolder, '1.19.4', 'vanilla', 'latest', null, true)

  return await invoke('plugin:import|import_instance', {
    profilePath,
    launcherType,
    basePath,
    instanceFolder,
  })
}

/// Checks if this instance is valid for importing, given a certain launcher type
/// eg: is_valid_importable_instance("C:/MultiMC/Instance 1", "MultiMC")
export async function is_valid_importable_instance(instanceFolder, launcherType) {
  return await invoke('plugin:import|is_valid_importable_instance', {
    instanceFolder,
    launcherType,
  })
}

/// Gets the default path for the given launcher type
/// null if it can't be found or doesn't exist
/// eg: get_default_launcher_path("MultiMC")
export async function get_default_launcher_path(launcherType) {
  return await invoke('plugin:import|get_default_launcher_path', { launcherType })
}
