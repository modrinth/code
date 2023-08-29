<template>
  <JavaDetectionModal ref="detectJavaModal" @submit="(val) => emit('update:modelValue', val)" />
  <div class="toggle-setting" :class="{ compact }">
    <input
      autocomplete="off"
      :disabled="props.disabled"
      :value="props.modelValue ? props.modelValue.path : ''"
      type="text"
      class="installation-input"
      :placeholder="placeholder ?? '/path/to/java'"
      @input="
        (val) => {
          emit('update:modelValue', {
            ...props.modelValue,
            path: val.target.value,
          })
        }
      "
    />
    <span class="installation-buttons">
      <Button :disabled="props.disabled" @click="autoDetect">
        <SearchIcon />
        Auto detect
      </Button>
      <Button :disabled="props.disabled" @click="handleJavaFileInput()">
        <FolderSearchIcon />
        Browse
      </Button>
      <Button v-if="testingJava" disabled> Testing... </Button>
      <Button v-else-if="testingJavaSuccess === true">
        <CheckIcon class="test-success" />
        Success
      </Button>
      <Button v-else-if="testingJavaSuccess === false">
        <XIcon class="test-fail" />
        Failed
      </Button>
      <Button v-else :disabled="props.disabled" @click="testJava">
        <PlayIcon />
        Test
      </Button>
    </span>
  </div>
</template>

<script setup>
import { Button, SearchIcon, PlayIcon, CheckIcon, XIcon, FolderSearchIcon } from 'omorphia'
import {
  find_jre_17_jres,
  find_jre_8_jres,
  get_jre,
  extract_version_from_string,
} from '@/helpers/jre.js'
import { ref } from 'vue'
import { open } from '@tauri-apps/api/dialog'
import JavaDetectionModal from '@/components/ui/JavaDetectionModal.vue'
import { mixpanel_track } from '@/helpers/mixpanel'
import { handleError } from '@/store/state.js'

const props = defineProps({
  version: {
    type: Number,
    required: false,
    default: null,
  },
  modelValue: {
    type: Object,
    required: true,
  },
  disabled: {
    type: Boolean,
    required: false,
    default: false,
  },
  placeholder: {
    type: String,
    required: false,
    default: null,
  },
  compact: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['update:modelValue'])

const testingJava = ref(false)
const testingJavaSuccess = ref(null)
async function testJava() {
  testingJava.value = true
  let result = await get_jre(props.modelValue ? props.modelValue.path : '')

  testingJava.value = false
  if (result) {
    let [majorVersion, minorVersion] = await extract_version_from_string(result.version)
    testingJavaSuccess.value = majorVersion == 1 && minorVersion == props.version
  } else {
    testingJavaSuccess.value = false
  }

  mixpanel_track('JavaTest', {
    path: props.modelValue ? props.modelValue.path : '',
    success: !!result,
  })

  setTimeout(() => {
    testingJavaSuccess.value = null
  }, 2000)
}

async function handleJavaFileInput() {
  let filePath = await open()

  if (filePath) {
    let result = await get_jre(filePath)
    if (!result) {
      result = {
        path: filePath,
        version: props.version.toString(),
        architecture: 'x86',
      }

      mixpanel_track('JavaManualSelect', {
        path: filePath,
        version: props.version,
      })
    }

    emit('update:modelValue', result)
  }
}

const detectJavaModal = ref(null)
async function autoDetect() {
  if (!props.compact) {
    detectJavaModal.value.show(props.version, props.modelValue)
  } else {
    if (props.version == 8) {
      let versions = await find_jre_8_jres().catch(handleError)
      if (versions.length > 0) {
        emit('update:modelValue', versions[0])
      }
    } else {
      let versions = await find_jre_17_jres().catch(handleError)
      if (versions.length > 0) {
        emit('update:modelValue', versions[0])
      }
    }
  }
}
</script>

<style lang="scss" scoped>
.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.toggle-setting {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;

  &.compact {
    flex-wrap: wrap;
  }
}

.installation-buttons {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
}

.test-success {
  color: var(--color-green);
}

.test-fail {
  color: var(--color-red);
}
</style>
