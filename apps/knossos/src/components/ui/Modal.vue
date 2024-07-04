<template>
  <div v-if="shown">
    <div
      :class="{
        shown: actuallyShown,
        noblur: !$orElse(cosmetics.advancedRendering, true),
      }"
      class="modal-overlay"
      @click="hide"
    />
    <div class="modal-container" :class="{ shown: actuallyShown }">
      <div class="modal-body">
        <div v-if="header" class="header">
          <strong>{{ header }}</strong>
          <button class="iconified-button icon-only transparent" @click="hide">
            <CrossIcon />
          </button>
        </div>
        <div class="content">
          <slot />
        </div>
      </div>
    </div>
  </div>
  <div v-else />
</template>

<script>
import CrossIcon from '~/assets/images/utils/x.svg?component'

export default {
  components: {
    CrossIcon,
  },
  props: {
    header: {
      type: String,
      default: null,
    },
  },
  setup() {
    const cosmetics = useCosmetics()

    return { cosmetics }
  },
  data() {
    return {
      shown: false,
      actuallyShown: false,
    }
  },
  methods: {
    show() {
      this.shown = true
      setTimeout(() => {
        this.actuallyShown = true
      }, 50)
    },
    hide() {
      this.actuallyShown = false
      setTimeout(() => {
        this.shown = false
      }, 300)
    },
  },
}
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

.modal-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 21;
  visibility: hidden;
  pointer-events: none;

  &.shown {
    visibility: visible;
    .modal-body {
      opacity: 1;
      visibility: visible;
      transform: translateY(0);
    }
  }

  .modal-body {
    position: fixed;
    box-shadow: var(--shadow-raised), var(--shadow-inset);
    border-radius: var(--size-rounded-lg);
    max-height: calc(100% - 2 * var(--spacing-card-bg));
    overflow-y: auto;
    width: 600px;
    pointer-events: auto;
    outline: 3px solid transparent;

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      background-color: var(--color-bg);
      padding: var(--spacing-card-md) var(--spacing-card-lg);

      strong {
        font-size: 1.25rem;
        margin: 0.67em 0;
      }
    }

    .content {
      background-color: var(--color-raised-bg);
    }

    transform: translateY(50vh);
    visibility: hidden;
    opacity: 0;
    transition: all 0.25s ease-in-out;

    @media screen and (max-width: 650px) {
      width: calc(100% - 2 * var(--spacing-card-bg));
    }
  }
}
</style>
