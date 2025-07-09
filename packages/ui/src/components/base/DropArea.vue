<template>
  <Teleport to="body">
    <div
      ref="dropAreaRef"
      class="drop-area"
      @drop.stop.prevent="handleDrop"
      @dragenter.prevent="allowDrag"
      @dragover.prevent="allowDrag"
      @dragleave.prevent="hideDropArea"
    />
  </Teleport>
  <slot />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const props = withDefaults(
  defineProps<{
    accept: string
  }>(),
  {
    accept: '*',
  },
)

const emit = defineEmits(['change'])

const dropAreaRef = ref<HTMLDivElement>()
const fileAllowed = ref(false)

const hideDropArea = () => {
  if (dropAreaRef.value) {
    dropAreaRef.value.style.visibility = 'hidden'
  }
}

const handleDrop = (event: DragEvent) => {
  hideDropArea()
  if (event.dataTransfer && event.dataTransfer.files && fileAllowed.value) {
    emit('change', event.dataTransfer.files)
  }
}

const allowDrag = (event: DragEvent) => {
  const file = event.dataTransfer?.items[0]
  if (
    file &&
    props.accept
      .split(',')
      .reduce((acc, t) => acc || file.type.startsWith(t) || file.type === t || t === '*', false)
  ) {
    fileAllowed.value = true
    event.dataTransfer.dropEffect = 'copy'
    event.preventDefault()
    if (dropAreaRef.value) {
      dropAreaRef.value.style.visibility = 'visible'
    }
  } else {
    fileAllowed.value = false
    hideDropArea()
  }
}

onMounted(() => {
  document.addEventListener('dragenter', allowDrag)
})
</script>

<style lang="scss" scoped>
.drop-area {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 10;
  visibility: hidden;
  background-color: hsla(0, 0%, 0%, 0.5);
  transition:
    visibility 0.2s ease-in-out,
    background-color 0.1s ease-in-out;
  display: flex;
  &::before {
    --indent: 4rem;
    content: ' ';
    position: relative;
    top: var(--indent);
    left: var(--indent);
    width: calc(100% - (2 * var(--indent)));
    height: calc(100% - (2 * var(--indent)));
    border-radius: 1rem;
    border: 0.25rem dashed var(--color-button-bg);
  }

  @media (prefers-reduced-motion) {
    transition: none !important;
  }
}
</style>
