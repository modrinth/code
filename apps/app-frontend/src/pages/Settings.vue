<script setup>
import { ref, watch, onMounted } from 'vue'
import { LogOutIcon, LogInIcon, BoxIcon, FolderSearchIcon, TrashIcon } from '@modrinth/assets'
import { Card, Slider, DropdownSelect, Toggle, Button } from '@modrinth/ui'
import { handleError, useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { get_java_versions, get_max_memory, set_java_version } from '@/helpers/jre'
import { get as getCreds, logout } from '@/helpers/mr_auth.js'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import ModrinthLoginScreen from '@/components/ui/tutorial/ModrinthLoginScreen.vue'
import { optOutAnalytics, optInAnalytics } from '@/helpers/analytics'
import { open } from '@tauri-apps/plugin-dialog'
import { getOS } from '@/helpers/utils.js'
import { getVersion } from '@tauri-apps/api/app'
import { get_user, purge_cache_types } from '@/helpers/cache.js'
import { hide_ads_window } from '@/helpers/ads.js'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'

onMounted(() => {
  hide_ads_window(true)
})

const pageOptions = ['Home', 'Library']

const themeStore = useTheming()

const version = await getVersion()

const accessSettings = async () => {
  const settings = await get()

  settings.launchArgs = settings.extra_launch_args.join(' ')
  settings.envVars = settings.custom_env_vars.map((x) => x.join('=')).join(' ')

  return settings
}

const fetchSettings = await accessSettings().catch(handleError)

const settings = ref(fetchSettings)

const maxMemory = ref(Math.floor((await get_max_memory().catch(handleError)) / 1024))

watch(
  settings,
  async (oldSettings, newSettings) => {
    if (oldSettings.loaded_config_dir !== newSettings.loaded_config_dir) {
      return
    }

    const setSettings = JSON.parse(JSON.stringify(newSettings))

    if (setSettings.telemetry) {
      optInAnalytics()
    } else {
      optOutAnalytics()
    }

    setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
    setSettings.custom_env_vars = setSettings.envVars
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map((x) => x.split('=').filter(Boolean))

    if (!setSettings.hooks.pre_launch) {
      setSettings.hooks.pre_launch = null
    }
    if (!setSettings.hooks.wrapper) {
      setSettings.hooks.wrapper = null
    }
    if (!setSettings.hooks.post_exit) {
      setSettings.hooks.post_exit = null
    }

    if (!setSettings.custom_dir) {
      setSettings.custom_dir = null
    }

    await set(setSettings)
  },
  { deep: true },
)

const javaVersions = ref(await get_java_versions().catch(handleError))
async function updateJavaVersion(version) {
  if (version?.path === '') {
    version.path = undefined
  }

  if (version?.path) {
    version.path = version.path.replace('java.exe', 'javaw.exe')
  }

  await set_java_version(version).catch(handleError)
}

async function fetchCredentials() {
  const creds = await getCreds().catch(handleError)
  if (creds && creds.user_id) {
    creds.user = await get_user(creds.user_id).catch(handleError)
  }
  credentials.value = creds
}

const credentials = ref()
await fetchCredentials()

const loginScreenModal = ref()

async function logOut() {
  await logout().catch(handleError)
  await fetchCredentials()
}

async function signInAfter() {
  await fetchCredentials()
}

async function findLauncherDir() {
  const newDir = await open({
    multiple: false,
    directory: true,
    title: 'Select a new app directory',
  })

  if (newDir) {
    settings.value.custom_dir = newDir
  }
}

async function purgeCache() {
  await purge_cache_types([
    'project',
    'version',
    'user',
    'team',
    'organization',
    'loader_manifest',
    'minecraft_manifest',
    'categories',
    'report_types',
    'loaders',
    'game_versions',
    'donation_platforms',
    'file_update',
    'search_results',
  ]).catch(handleError)
}
</script>

<template>
  <div class="settings-page">
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">General settings</span>
        </h3>
      </div>
      <ModrinthLoginScreen ref="loginScreenModal" :callback="signInAfter" />
      <div class="adjacent-input">
        <label for="sign-in">
          <span class="label__title">Manage account</span>
          <span v-if="credentials" class="label__description">
            You are currently logged in as {{ credentials.user.username }}.
          </span>
          <span v-else> Sign in to your Modrinth account. </span>
        </label>
        <button v-if="credentials" id="sign-in" class="btn" @click="logOut">
          <LogOutIcon />
          Sign out
        </button>
        <button v-else id="sign-in" class="btn" @click="$refs.loginScreenModal.show()">
          <LogInIcon />
          Sign in
        </button>
      </div>
      <ConfirmModalWrapper
        ref="purgeCacheConfirmModal"
        title="Are you sure you want to purge the cache?"
        description="If you proceed, your entire cache will be purged. This may slow down the app temporarily."
        :has-to-type="false"
        proceed-label="Purge cache"
        @proceed="purgeCache"
      />
      <div class="adjacent-input">
        <label for="purge-cache">
          <span class="label__title">App cache</span>
          <span class="label__description">
            The Modrinth app stores a cache of data to speed up loading. This can be purged to force
            the app to reload data. <br />
            This may slow down the app temporarily.
          </span>
        </label>
        <button id="purge-cache" class="btn" @click="$refs.purgeCacheConfirmModal.show()">
          <TrashIcon />
          Purge cache
        </button>
      </div>
      <label for="appDir">
        <span class="label__title">App directory</span>
        <span class="label__description">
          The directory where the launcher stores all of its files. Changes will be applied after
          restarting the launcher.
        </span>
      </label>
      <div class="app-directory">
        <div class="iconified-input">
          <BoxIcon />
          <input id="appDir" v-model="settings.custom_dir" type="text" class="input" />
          <Button class="r-btn" @click="findLauncherDir">
            <FolderSearchIcon />
          </Button>
        </div>
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Display</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title">Color theme</span>
          <span class="label__description">Change the global launcher color theme.</span>
        </label>
        <DropdownSelect
          id="theme"
          name="Theme dropdown"
          :options="themeStore.themeOptions"
          :default-value="settings.theme"
          :model-value="settings.theme"
          class="theme-dropdown"
          @change="
            (e) => {
              themeStore.setThemeState(e.option.toLowerCase())
              settings.theme = themeStore.selectedTheme
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="advanced-rendering">
          <span class="label__title">Advanced rendering</span>
          <span class="label__description">
            Enables advanced rendering such as blur effects that may cause performance issues
            without hardware-accelerated rendering.
          </span>
        </label>
        <Toggle
          id="advanced-rendering"
          :model-value="themeStore.advancedRendering"
          :checked="themeStore.advancedRendering"
          @update:model-value="
            (e) => {
              themeStore.advancedRendering = e
              settings.advanced_rendering = themeStore.advancedRendering
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="minimize-launcher">
          <span class="label__title">Minimize launcher</span>
          <span class="label__description"
            >Minimize the launcher when a Minecraft process starts.</span
          >
        </label>
        <Toggle
          id="minimize-launcher"
          :model-value="settings.hide_on_process_start"
          :checked="settings.hide_on_process_start"
          @update:model-value="
            (e) => {
              settings.hide_on_process_start = e
            }
          "
        />
      </div>
      <div v-if="getOS() != 'MacOS'" class="adjacent-input">
        <label for="native-decorations">
          <span class="label__title">Native decorations</span>
          <span class="label__description">Use system window frame (app restart required).</span>
        </label>
        <Toggle
          id="native-decorations"
          :model-value="settings.native_decorations"
          :checked="settings.native_decorations"
          @update:model-value="
            (e) => {
              settings.native_decorations = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="opening-page">
          <span class="label__title">Default landing page</span>
          <span class="label__description">Change the page to which the launcher opens on.</span>
        </label>
        <DropdownSelect
          id="opening-page"
          name="Opening page dropdown"
          :options="pageOptions"
          :default-value="settings.default_page"
          :model-value="settings.default_page"
          class="opening-page"
          @change="
            (e) => {
              settings.default_page = e.option
            }
          "
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Resource management</span>
        </h3>
      </div>

      <div class="adjacent-input">
        <label for="max-downloads">
          <span class="label__title">Maximum concurrent downloads</span>
          <span class="label__description">
            The maximum amount of files the launcher can download at the same time. Set this to a
            lower value if you have a poor internet connection. (app restart required to take
            effect)
          </span>
        </label>
        <Slider
          id="max-downloads"
          v-model="settings.max_concurrent_downloads"
          :min="1"
          :max="10"
          :step="1"
        />
      </div>

      <div class="adjacent-input">
        <label for="max-writes">
          <span class="label__title">Maximum concurrent writes</span>
          <span class="label__description">
            The maximum amount of files the launcher can write to the disk at once. Set this to a
            lower value if you are frequently getting I/O errors. (app restart required to take
            effect)
          </span>
        </label>
        <Slider
          id="max-writes"
          v-model="settings.max_concurrent_writes"
          :min="1"
          :max="50"
          :step="1"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Privacy</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="opt-out-analytics">
          <span class="label__title">Personalized ads</span>
          <span class="label__description">
            Modrinth's ad provider, Aditude, shows ads based on your preferences. By disabling this
            option, you opt out and ads will no longer be shown based on your interests.
          </span>
        </label>
        <Toggle
          id="opt-out-analytics"
          :model-value="settings.personalized_ads"
          :checked="settings.personalized_ads"
          @update:model-value="
            (e) => {
              settings.personalized_ads = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="opt-out-analytics">
          <span class="label__title">Telemetry</span>
          <span class="label__description">
            Modrinth collects anonymized analytics and usage data to improve our user experience and
            customize your experience. By disabling this option, you opt out and your data will no
            longer be collected.
          </span>
        </label>
        <Toggle
          id="opt-out-analytics"
          :model-value="settings.telemetry"
          :checked="settings.telemetry"
          @update:model-value="
            (e) => {
              settings.telemetry = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="disable-discord-rpc">
          <span class="label__title">Discord RPC</span>
          <span class="label__description">
            Manages the Discord Rich Presence integration. Disabling this will cause 'Modrinth' to
            no longer show up as a game or app you are using on your Discord profile. This does not
            disable any instance-specific Discord Rich Presence integrations, such as those added by
            mods. (app restart required to take effect)
          </span>
        </label>
        <Toggle
          id="disable-discord-rpc"
          v-model="settings.discord_rpc"
          :checked="settings.discord_rpc"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Java settings</span>
        </h3>
      </div>
      <template v-for="javaVersion in [21, 17, 8]" :key="`java-${javaVersion}`">
        <label :for="'java-' + javaVersion">
          <span class="label__title">Java {{ javaVersion }} location</span>
        </label>
        <JavaSelector
          :id="'java-selector-' + javaVersion"
          v-model="javaVersions[javaVersion]"
          :version="javaVersion"
          @update:model-value="updateJavaVersion"
        />
      </template>
      <hr class="card-divider" />
      <label for="java-args">
        <span class="label__title">Java arguments</span>
      </label>
      <input
        id="java-args"
        v-model="settings.launchArgs"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter java arguments..."
      />
      <label for="env-vars">
        <span class="label__title">Environmental variables</span>
      </label>
      <input
        id="env-vars"
        v-model="settings.envVars"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter environmental variables..."
      />
      <hr class="card-divider" />
      <div class="adjacent-input">
        <label for="max-memory">
          <span class="label__title">Java memory</span>
          <span class="label__description">
            The memory allocated to each instance when it is ran.
          </span>
        </label>
        <Slider
          id="max-memory"
          v-model="settings.memory.maximum"
          :min="8"
          :max="maxMemory"
          :step="64"
          unit="mb"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Hooks</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="pre-launch">
          <span class="label__title">Pre launch</span>
          <span class="label__description"> Ran before the instance is launched. </span>
        </label>
        <input
          id="pre-launch"
          v-model="settings.hooks.pre_launch"
          autocomplete="off"
          type="text"
          placeholder="Enter pre-launch command..."
        />
      </div>
      <div class="adjacent-input">
        <label for="wrapper">
          <span class="label__title">Wrapper</span>
          <span class="label__description"> Wrapper command for launching Minecraft. </span>
        </label>
        <input
          id="wrapper"
          v-model="settings.hooks.wrapper"
          autocomplete="off"
          type="text"
          placeholder="Enter wrapper command..."
        />
      </div>
      <div class="adjacent-input">
        <label for="post-exit">
          <span class="label__title">Post exit</span>
          <span class="label__description"> Ran after the game closes. </span>
        </label>
        <input
          id="post-exit"
          v-model="settings.hooks.post_exit"
          autocomplete="off"
          type="text"
          placeholder="Enter post-exit command..."
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Window size</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="fullscreen">
          <span class="label__title">Fullscreen</span>
          <span class="label__description">
            Overwrites the options.txt file to start in full screen when launched.
          </span>
        </label>
        <Toggle
          id="fullscreen"
          :model-value="settings.force_fullscreen"
          :checked="settings.force_fullscreen"
          @update:model-value="
            (e) => {
              settings.force_fullscreen = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="width">
          <span class="label__title">Width</span>
          <span class="label__description"> The width of the game window when launched. </span>
        </label>
        <input
          id="width"
          v-model="settings.game_resolution[0]"
          :disabled="settings.force_fullscreen"
          autocomplete="off"
          type="number"
          placeholder="Enter width..."
        />
      </div>
      <div class="adjacent-input">
        <label for="height">
          <span class="label__title">Height</span>
          <span class="label__description"> The height of the game window when launched. </span>
        </label>
        <input
          id="height"
          v-model="settings.game_resolution[1]"
          :disabled="settings.force_fullscreen"
          autocomplete="off"
          type="number"
          class="input"
          placeholder="Enter height..."
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">About</span>
        </h3>
      </div>
      <div>
        <label>
          <span class="label__title">App version</span>
          <span class="label__description">Modrinth App v{{ version }} </span>
        </label>
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.settings-page {
  margin: 1rem;
}

.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.theme-dropdown {
  text-transform: capitalize;
}

.card-divider {
  margin: 1rem 0;
}

.app-directory {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);

  .iconified-input {
    flex-grow: 1;

    input {
      flex-basis: auto;
    }
  }
}
</style>
