<script setup>
import { ref } from 'vue'
import { get_java_versions, set_java_version } from '@/helpers/jre'
import { handleError } from '@/store/notifications'
import JavaSelector from '@/components/ui/JavaSelector.vue'

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
</script>
<template>
  <div v-for="(javaVersion, index) in [21, 17, 8]" :key="`java-${javaVersion}`">
    <h2 class="m-0 text-lg font-extrabold text-contrast" :class="{ 'mt-4': index !== 0 }">
      Java {{ javaVersion }} location
    </h2>
    <JavaSelector
      :id="'java-selector-' + javaVersion"
      v-model="javaVersions[javaVersion]"
      :version="javaVersion"
      @update:model-value="updateJavaVersion"
    />
  </div>
</template>
