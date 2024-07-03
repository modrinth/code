<template>
  <div
    ref="drop_area"
    class="drop-area"
    @drop.stop.prevent="
      (event) => {
        $refs.drop_area.style.visibility = 'hidden'

        if (event.dataTransfer && event.dataTransfer.files && fileAllowed) {
          $emit('change', event.dataTransfer.files)
        }
      }
    "
    @dragenter.prevent="allowDrag"
    @dragover.prevent="allowDrag"
    @dragleave.prevent="$refs.drop_area.style.visibility = 'hidden'"
  />
</template>

<script>
export default {
  props: {
    accept: {
      type: String,
      default: '',
    },
  },
  emits: ['change'],
  data() {
    return {
      fileAllowed: false,
    }
  },
  mounted() {
    document.addEventListener('dragenter', this.allowDrag)
  },
  methods: {
    allowDrag(event) {
      const file = event.dataTransfer?.items[0]

      if (
        file &&
        this.accept
          .split(',')
          .reduce((acc, t) => acc || file.type.startsWith(t) || file.type === t || t === '*', false)
      ) {
        this.fileAllowed = true
        event.dataTransfer.dropEffect = 'copy'
        event.preventDefault()

        if (this.$refs.drop_area) {
          this.$refs.drop_area.style.visibility = 'visible'
        }
      } else {
        this.fileAllowed = false

        if (this.$refs.drop_area) {
          this.$refs.drop_area.style.visibility = 'hidden'
        }
      }
    },
  },
}
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
  transition: visibility 0.2s ease-in-out, background-color 0.1s ease-in-out;
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
}
</style>
