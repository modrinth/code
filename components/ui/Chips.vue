<template>
  <div class="chips">
    <button
      v-for="item in items"
      :key="item"
      class="iconified-button"
      :class="{ selected: selected === item }"
      @click="toggleItem(item)"
    >
      <CheckIcon v-if="selected === item" />
      <span>{{ formatLabel(item) }}</span>
    </button>
  </div>
</template>

<script>
import CheckIcon from '~/assets/images/utils/check.svg?inline'

export default {
  name: 'Chips',
  components: {
    CheckIcon,
  },
  props: {
    value: {
      required: true,
      type: String,
    },
    items: {
      required: true,
      type: Array,
    },
    neverEmpty: {
      default: true,
      type: Boolean,
    },
    formatLabel: {
      default: (x) => x,
      type: Function,
    },
  },
  computed: {
    selected: {
      get() {
        return this.value
      },
      set(value) {
        this.$emit('input', value)
      },
    },
  },
  created() {
    if (this.items.length > 0 && this.neverEmpty) {
      this.selected = this.items[0]
    }
  },
  methods: {
    toggleItem(item) {
      if (this.selected === item && !this.neverEmpty) {
        this.selected = null
      } else {
        this.selected = item
      }
    },
  },
}
</script>

<style lang="scss" scoped>
.chips {
  display: flex;
  grid-gap: 0.5rem;
  flex-wrap: wrap;

  .iconified-button {
    text-transform: capitalize;

    svg {
      width: 1em;
      height: 1em;
    }

    &:focus-visible {
      outline: 0.25rem solid #ea80ff;
      border-radius: 0.25rem;
    }
  }

  .selected {
    color: var(--color-button-text-active);
    background-color: var(--color-brand-highlight);
    box-shadow: inset 0 0 0 transparent, 0 0 0 2px var(--color-brand);
  }
}
</style>
