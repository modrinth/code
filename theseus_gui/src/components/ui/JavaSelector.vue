<template>
  <JavaDetectionModal ref="detectJavaModal" @submit="(val) => emit('update:modelValue', val)" />
  <div class="toggle-setting">
    <input
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
      <Button
        :disabled="props.disabled"
        @click="$refs.detectJavaModal.show(props.version, props.modelValue)"
      >
        <SearchIcon />
        Auto detect
      </Button>
      <Button :disabled="props.disabled" @click="handleJavaFileInput()">
        <FolderSearchIcon />
        Browse
      </Button>
      <Button :disabled="props.disabled" @click="testJava">
        <PlayIcon />
        Test
      </Button>
      <AnimatedLogo v-if="testingJava === true" class="testing-loader" />
      <CheckIcon
        v-else-if="testingJavaSuccess === true && testingJava === false"
        class="test-success"
      />
      <XIcon v-else-if="testingJavaSuccess === false && testingJava === false" class="test-fail" />
    </span>
  </div>
</template>

<script setup>
import {
  Button,
  SearchIcon,
  PlayIcon,
  CheckIcon,
  XIcon,
  AnimatedLogo,
  FolderSearchIcon,
} from 'omorphia'
import { get_jre } from '@/helpers/jre.js'
import { ref } from 'vue'
import { open } from '@tauri-apps/api/dialog'
import JavaDetectionModal from '@/components/ui/JavaDetectionModal.vue'

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
})

const emit = defineEmits(['update:modelValue'])

const testingJava = ref(false)
const testingJavaSuccess = ref(null)
async function testJava() {
  testingJava.value = true
  let result = await get_jre(props.modelValue ? props.modelValue.path : '')
  testingJava.value = false
  testingJavaSuccess.value = !!result

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
    }

    emit('update:modelValue', result)
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
}

.installation-buttons {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
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
<style lang="scss">
.testing-loader {
  height: 1rem !important;
  width: 1rem !important;

  svg {
    height: inherit !important;
    width: inherit !important;
  }
}
</style>
