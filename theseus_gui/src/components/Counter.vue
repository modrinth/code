<script setup>
import { Button } from 'omorphia'
import { useStore } from 'vuex'

const { state, commit } = useStore()
</script>

<template>
  <div>
    <p>Increment counter: {{ state.count }}</p>
    <Button @click="commit('increment')">Increment</Button>
    <Button @click="commit('decrement')">Decrement</Button>
  </div>
</template>
