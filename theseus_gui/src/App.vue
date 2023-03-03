<script setup>
import { RouterLink, RouterView } from 'vue-router'

import Counter from './components/Counter.vue'
</script>

<template>
  <div class="container">
    <RouterLink to="/">Index</RouterLink>
    <RouterLink to="/about">About</RouterLink>
    <h1>Modrinth</h1>
    <Counter />
    <RouterView></RouterView>
  </div>
</template>

<style scoped></style>
