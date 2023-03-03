<script setup>
import { Button } from 'omorphia'

import { ref } from 'vue'
const count = ref(0)
</script>

<template>
  <div>
    <p>Increment counter: {{ count }}</p>
    <Button @click="count++">Increment</Button>
    <Button @click="count--">Decrement</Button>
  </div>
</template>
