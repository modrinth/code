<script setup>
import Instance from './ui/Instance.vue'

const props = defineProps({
  instances: Object,
  label: String,
})

console.log(props.instances)
</script>

<template>
  <div class="mod-row">
    <div class="header">
      <p>{{ props.label }}</p>
      <div aria-hidden="true"></div>
    </div>
    <section class="mods">
      <Instance v-for="instance in props.instances" display="gallery" :instance="instance" />
    </section>
  </div>
</template>

<style lang="scss" scoped>
.mod-row {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  width: 100%;
  margin-top: 2rem;

  .header {
    display: flex;
    justify-content: space-between;
    align-items: inherit;
    width: 90%;

    p {
      font-size: 14px;
    }

    div {
      background: #fff;
      height: 1px;
      width: 80%;
      border-radius: 5px;
    }
  }

  .mods {
    display: flex;
    justify-content: space-evenly;
    align-items: inherit;
  }
}
</style>
