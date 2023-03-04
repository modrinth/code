<script setup>
const props = defineProps({
  display: String,
  instance: Object,
})

const handleInstanceClick = (id) => {
  console.log(id)
}
</script>

<template>
  <div>
    <div
      @click="handleInstanceClick(props.instance.id)"
      class="instance-list-item"
      v-if="display === 'list'"
    >
      <p>{{ props.instance.name }}</p>
    </div>
    <div class="instance-gallery-item" v-else-if="display === 'gallery'">
      <p>{{ props.instance.name }}</p>
      <p>{{ props.instance.version }}</p>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.instance-list-item {
  margin: 5px auto;
  cursor: pointer;
  transition: all ease-out 0.1s;

  &:hover {
    font-weight: bold;
  }
}

.instance-gallery-item {
  width: 100px;
  height: 100px;
  background: #fff;
  margin: 20px;
}
</style>
