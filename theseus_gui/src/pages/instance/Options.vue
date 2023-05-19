<template>
  <section class="card">
    <div class="label">
      <h3>
        <span class="label__title size-card-header">Profile</span>
      </h3>
    </div>
    <label for="instance-icon">
      <span class="label__title">Icon</span>
    </label>
    <div class="input-group">
      <Avatar
        :src="
          !instance.metadata.icon ||
          (instance.metadata.icon && instance.metadata.icon.startsWith('http'))
            ? instance.metadata.icon
            : convertFileSrc(instance.metadata?.icon)
        "
        size="md"
        class="project__icon"
      />
      <div class="input-stack">
        <Button id="instance-icon" class="btn">
          <UploadIcon />
          Upload icon
        </Button>
        <button class="btn">
          <TrashIcon />
          Remove icon
        </button>
      </div>
    </div>

    <label for="project-name">
      <span class="label__title">Name</span>
    </label>
    <input id="profile-name" v-model="title" maxlength="80" type="text" />

    <div class="adjacent-input">
      <label for="edit-versions">
        <span class="label__title">Edit mod loader/game versions</span>
        <span class="label__description">
          Allows you to change the mod loader, loader version, or game version of the profile.
        </span>
      </label>
      <button id="edit-versions" class="btn">
        <EditIcon />
        Edit versions
      </button>
    </div>
  </section>
  <Card class="settings-card">
    <h2 class="settings-title">Java</h2>
    <div class="settings-group">
      <h3>Installation</h3>
      <Checkbox v-model="overrideJavaInstall" label="Override global java installations" />
      <JavaSelector v-model="javaInstall" :disabled="!overrideJavaInstall" />
    </div>
    <hr class="card-divider" />
    <div class="settings-group">
      <h3>Java arguments</h3>
      <Checkbox v-model="overrideJavaArgs" label="Override global java arguments" />
      <input
        v-model="javaArgs"
        :disabled="!overrideJavaArgs"
        type="text"
        class="input installation-input"
        placeholder="Enter java arguments..."
      />
    </div>
    <div class="settings-group">
      <h3>Environment variables</h3>
      <Checkbox v-model="overrideEnvVars" label="Override global environment variables" />
      <input
        v-model="envVars"
        :disabled="!overrideEnvVars"
        type="text"
        class="input installation-input"
        placeholder="Enter environment variables..."
      />
    </div>
    <hr class="card-divider" />
    <div class="settings-group">
      <Checkbox v-model="overrideMemorySettings" label="Override global memory settings" />
      <div class="sliders">
        <span class="slider">
          Minimum memory
          <Slider
            v-model="memory.minimum"
            :disabled="!overrideMemorySettings"
            :min="256"
            :max="maxMemory"
            :step="10"
          />
        </span>
        <span class="slider">
          Maximum memory
          <Slider
            v-model="memory.maximum"
            :disabled="!overrideMemorySettings"
            :min="256"
            :max="maxMemory"
            :step="10"
          />
        </span>
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Window</h2>
    <Checkbox v-model="overrideWindowSettings" label="Override global window settings" />
    <div class="settings-group">
      <div class="toggle-setting">
        Width
        <input
          v-model="resolution[0]"
          :disabled="!overrideWindowSettings"
          type="number"
          class="input"
        />
      </div>
      <div class="toggle-setting">
        Height
        <input
          v-model="resolution[1]"
          :disabled="!overrideWindowSettings"
          type="number"
          class="input"
        />
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Hooks</h2>
    <Checkbox v-model="overrideHooks" label="Override global hooks" />
    <div class="settings-group">
      <div class="toggle-setting">
        Pre launch
        <input v-model="hooks.pre_launch" :disabled="!overrideHooks" type="text" class="input" />
      </div>
      <div class="toggle-setting">
        Wrapper
        <input v-model="hooks.wrapper" :disabled="!overrideHooks" type="text" class="input" />
      </div>
      <div class="toggle-setting">
        Post exit
        <input v-model="hooks.post_exit" :disabled="!overrideHooks" type="text" class="input" />
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Profile management</h2>
    <div class="settings-group">
      <div class="toggle-setting">
        Repair profile
        <button class="btn btn-highlight" :disabled="repairing" @click="repairProfile">
          <HammerIcon /> Repair
        </button>
      </div>
      <div class="toggle-setting">
        Delete profile
        <button class="btn btn-danger" :disabled="removing" @click="removeProfile">
          <TrashIcon /> Delete
        </button>
      </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Slider, TrashIcon, Checkbox, UploadIcon, Avatar, EditIcon } from 'omorphia'
import { HammerIcon } from '@/assets/icons'
import { useRouter } from 'vue-router'
import { get_optimal_jre_key, install, remove } from '@/helpers/profile.js'
import { readonly, ref } from 'vue'
import { get_max_memory } from '@/helpers/jre.js'
import { get } from '@/helpers/settings.js'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'

const router = useRouter()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

const title = ref(props.instance.metadata.name)
// const icon = ref(props.instance.metadata.icon)

const globalSettings = await get()

const overrideJavaInstall = ref(!!props.instance.java?.override_version)
const optimalJava = readonly(await get_optimal_jre_key(props.instance.path))
const javaInstall = ref(
  optimalJava ?? props.instance.profile.java?.override_version ?? { path: '', version: '' }
)

const overrideJavaArgs = ref(!!props.instance.java?.extra_arguments)
const javaArgs = ref(props.instance.java?.extra_arguments ?? globalSettings.custom_java_args)

const overrideEnvVars = ref(!!props.instance.java?.custom_env_args)
const envVars = ref(props.instance.java?.custom_env_args ?? globalSettings.custom_env_args)

const overrideMemorySettings = ref(!!props.instance.memory)
const memory = ref(props.instance.memory ?? globalSettings.memory)
const maxMemory = readonly((await get_max_memory()) / 1024)

const overrideWindowSettings = ref(!!props.instance.resolution)
const resolution = ref(props.instance.resolution ?? globalSettings.game_resolution)

const overrideHooks = ref(!!props.instance.hooks)
const hooks = ref(props.instance.hooks ?? globalSettings.hooks)

const repairing = ref(false)

// async function updateProfile() {
//   const editProfile = {
//     title: title.value
//   }
//
//   if (overrideJavaInstall.value) {
//     if (javaInstall.value.path !== '') {
//       editProfile.java = {
//         override_version: javaInstall.value,
//         ...editProfile.java
//       }
//     }
//   }
//
//   if (overrideJavaArgs.value) {
//     if (javaArgs.value !== )
//   }
//
//   await edit(props.instance.path, editProfile)
// }

async function repairProfile() {
  repairing.value = true
  await install(props.instance.path)
  repairing.value = false
}

const removing = ref(false)
async function removeProfile() {
  removing.value = true
  await remove(props.instance.path)
  removing.value = false

  await router.push({ path: '/' })
}
</script>

<style scoped lang="scss">
.settings-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

.input-stack {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.settings-title {
  color: var(--color-contrast);
}

.settings-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.installation-input {
  width: 100%;
}

.sliders {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 1rem;
  width: 100%;

  .slider {
    flex-grow: 1;
  }
}

.toggle-setting {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.card-divider {
  background-color: var(--color-button-bg);
  border: none;
  color: var(--color-button-bg);
  height: 1px;
  margin: var(--gap-sm) 0;
}

:deep(button.checkbox) {
  border: none;
}
</style>
