<template>
  <div class="chips">
    <Button
      v-for="item in items"
      :key="item"
      class="btn"
      :class="{ selected: selected === item, capitalize: capitalize }"
      @click="toggleItem(item)"
    >
      <CheckIcon v-if="selected === item" />
      <span>{{ formatLabel(item) }}</span>
    </Button>
  </div>
</template>
<script setup>
import { CheckIcon } from '@modrinth/assets'
</script>
<script>
import { defineComponent } from 'vue'
import Button from './Button.vue'

export default defineComponent({
  props: {
    modelValue: {
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
    capitalize: {
      type: Boolean,
      default: true,
    },
  },
  emits: ['update:modelValue'],
  computed: {
    selected: {
      get() {
        return this.modelValue
      },
      set(value) {
        this.$emit('update:modelValue', value)
      },
    },
  },
  created() {
    if (this.items.length > 0 && this.neverEmpty && !this.modelValue) {
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
})
</script>

<style lang="scss" scoped>
.chips {
  display: flex;
  grid-gap: 0.5rem;
  flex-wrap: wrap;

  .btn {
    &.capitalize {
      text-transform: capitalize;
    }

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
    color: var(--color-contrast);
    background-color: var(--color-brand-highlight);
    box-shadow:
      inset 0 0 0 transparent,
      0 0 0 2px var(--color-brand);
  }
}
</style>
