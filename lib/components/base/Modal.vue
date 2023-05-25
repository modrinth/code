<template>
  <div>
    <div
      :class="{
        shown: shown,
        noblur: noblur,
      }"
      class="modal-overlay"
      @click="hide"
    />
    <div class="modal-body" :class="{ shown: shown }">
      <div v-if="header" class="header">
        <h1>{{ header }}</h1>
        <button class="btn icon-only transparent" @click="hide">
          <XIcon />
        </button>
      </div>
      <div class="content">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup>
import { XIcon } from '@/components'
</script>
<script>
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    header: {
      type: String,
      default: null,
    },
    noblur: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      shown: false,
    }
  },
  methods: {
    show() {
      this.shown = true
    },
    hide() {
      this.shown = false
    },
  },
})
</script>

<style lang="scss" scoped>
.modal-overlay {
  visibility: hidden;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 20;

  transition: all 0.3s ease-in-out;

  &.shown {
    opacity: 1;
    visibility: visible;
    background: hsla(0, 0%, 0%, 0.5);
    backdrop-filter: blur(3px);
  }

  &.noblur {
    backdrop-filter: none;
  }
}

.modal-body {
  position: fixed;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 21;
  box-shadow: var(--shadow-raised), var(--shadow-inset);
  border-radius: var(--radius-lg);
  max-height: calc(100% - 2 * var(--gap-lg));
  overflow-y: auto;
  width: 600px;

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: var(--color-bg);
    padding: var(--gap-md) var(--gap-lg);

    h1 {
      font-size: 1.25rem;
      color: var(--color-contrast);
      font-weight: bolder;
    }
  }

  .content {
    background-color: var(--color-raised-bg);
  }

  top: calc(100% + 400px);
  visibility: hidden;
  opacity: 0;
  transition: all 0.25s ease-in-out;

  &.shown {
    opacity: 1;
    visibility: visible;
    top: 50%;
  }

  @media screen and (max-width: 650px) {
    width: calc(100% - 2 * var(--gap-lg));
  }
}
</style>
