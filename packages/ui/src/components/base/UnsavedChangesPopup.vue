<script setup lang="ts" generic="T">
import { HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

import ButtonStyled from './ButtonStyled.vue'

const emit = defineEmits<{
  (e: 'reset' | 'save', event: MouseEvent): void
}>()

const props = withDefaults(
  defineProps<{
    canReset?: boolean
    original: T
    modified: Partial<T>
    saving?: boolean
  }>(),
  {
    canReset: true,
    saving: false,
  },
)

const shown = computed(() => {
  let changed = false
  for (const key of Object.keys(props.modified)) {
    if (props.original[key] !== props.modified[key]) {
      changed = true
    }
  }
  return changed
})
</script>

<template>
  <Transition name="pop-in">
    <div v-if="shown" class="fixed w-full z-10 left-0 bottom-0 p-4">
      <div
        class="flex items-center rounded-2xl bg-bg-raised border-2 border-divider border-solid mx-auto max-w-[77rem] p-4"
      >
        <p class="m-0 font-semibold">You have unsaved changes.</p>
        <div class="ml-auto flex gap-2">
          <ButtonStyled type="transparent">
            <button :disabled="saving" @click="(e) => emit('reset', e)">
              <HistoryIcon /> Reset
            </button>
          </ButtonStyled>
          <ButtonStyled color="brand">
            <button :disabled="saving" @click="(e) => emit('save', e)">
              <SpinnerIcon v-if="saving" class="animate-spin" />
              <SaveIcon v-else />
              {{ saving ? 'Saving' : 'Save' }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.pop-in-enter-active {
  transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
}

.pop-in-leave-active {
  transition: all 0.25s ease;
}

.pop-in-enter-from {
  scale: 0.5;
  translate: 0 10rem;
  opacity: 0;
}

.pop-in-leave-to {
  scale: 0.96;
  translate: 0 0.25rem;
  opacity: 0;
}
</style>
