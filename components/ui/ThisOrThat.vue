<template>
  <div v-if="items.length !== 1" class="styled-tabs">
    <button
      v-for="item in items"
      :key="item"
      class="tab"
      :class="{ selected: selected === item }"
      @click="toggleItem(item)"
    >
      <span>{{ item }}</span>
    </button>
  </div>
</template>

<script>
export default {
  name: 'ThisOrThat',
  props: {
    items: {
      required: true,
      type: Array,
    },
  },
  data() {
    return {
      selected: '',
    }
  },
  created() {
    if (this.items.length > 0) {
      this.selected = this.items[0]
      this.$emit('input', this.selected)
    }
  },
  methods: {
    toggleItem(item) {
      this.selected = item
      this.$emit('input', item)
    },
  },
}
</script>

<style scoped>
button {
  text-transform: capitalize;
  margin: 0;
  padding: 0;
  background-color: transparent;
  border-radius: 0;
  color: inherit;
}

button span::first-letter {
  text-transform: uppercase;
}
</style>
