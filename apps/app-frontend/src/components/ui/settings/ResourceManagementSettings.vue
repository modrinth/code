<script setup>
import { Button, Slider } from '@modrinth/ui'
import { ref, watch } from 'vue'
import { get, set } from '@/helpers/settings.ts'
import { purge_cache_types } from '@/helpers/cache.js'
import { handleError } from '@/store/notifications.js'
import { BoxIcon, FolderSearchIcon, TrashIcon } from '@modrinth/assets'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { open } from '@tauri-apps/plugin-dialog'

const settings = ref(await get())

watch(
  settings,
  async () => {
    const setSettings = JSON.parse(JSON.stringify(settings.value))

    if (!setSettings.custom_dir) {
      setSettings.custom_dir = null
    }

    await set(setSettings)
  },
  { deep: true },
)

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
</script>

<template>
  <h2 class="m-0 text-lg font-extrabold text-contrast">App directory</h2>
  <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
    The directory where the launcher stores all of its files. Changes will be applied after
    restarting the launcher.
  </p>

  <div class="m-1 my-2">
    <div class="iconified-input w-full">
      <BoxIcon />
      <input id="appDir" v-model="settings.custom_dir" type="text" class="input" />
      <Button class="r-btn" @click="findLauncherDir">
        <FolderSearchIcon />
      </Button>
    </div>
  </div>

  <div>
    <ConfirmModalWrapper
      ref="purgeCacheConfirmModal"
      title="Are you sure you want to purge the cache?"
      description="If you proceed, your entire cache will be purged. This may slow down the app temporarily."
      :has-to-type="false"
      proceed-label="Purge cache"
      :show-ad-on-close="false"
      @proceed="purgeCache"
    />

    <h2 class="m-0 text-lg font-extrabold text-contrast">App cache</h2>
    <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
      The Modrinth app stores a cache of data to speed up loading. This can be purged to force the
      app to reload data. This may slow down the app temporarily.
    </p>
  </div>
  <button id="purge-cache" class="btn min-w-max" @click="$refs.purgeCacheConfirmModal.show()">
    <TrashIcon />
    Purge cache
  </button>

  <h2 class="m-0 text-lg font-extrabold text-contrast mt-4">Maximum concurrent downloads</h2>
  <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
    The maximum amount of files the launcher can download at the same time. Set this to a lower
    value if you have a poor internet connection. (app restart required to take effect)
  </p>
  <Slider
    id="max-downloads"
    v-model="settings.max_concurrent_downloads"
    :min="1"
    :max="10"
    :step="1"
  />

  <h2 class="mt-4 m-0 text-lg font-extrabold text-contrast">Maximum concurrent writes</h2>
  <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
    The maximum amount of files the launcher can write to the disk at once. Set this to a lower
    value if you are frequently getting I/O errors. (app restart required to take effect)
  </p>
  <Slider id="max-writes" v-model="settings.max_concurrent_writes" :min="1" :max="50" :step="1" />
</template>
