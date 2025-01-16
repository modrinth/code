<script setup>
const props = defineProps({
  data: {
    type: Object,
    default() {
      return {}
    },
  },
})

const emit = defineEmits(['setSkin'])
</script>

<template>
  <button class="button-base cursor-pointer bg-bg-raised rounded-2xl hover:brightness-12 flex flex-col items-center justify-center 5 p-0 py-4 border-0 active:scale-[0.96] transition-all w-full h-full" @click="emit('setSkin', props.data)">
    <img
      :src="props.data.model"
      alt="Mod card"
      class="max-w-[144px]"
      draggable="false"
    />
    <span class="m-0">{{ props.data.name }}</span>
  </button>
</template>
