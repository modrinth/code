<script setup>
import GridDisplay from '@/components/GridDisplay.vue'
import { shallowRef } from 'vue'
import { list } from '@/helpers/profile.js'

const profiles = await list()
console.log(profiles)
const instances = shallowRef(Object.values(profiles))
</script>

<template>
  <div>
    <GridDisplay label="Instances" :instances="instances" />
    <GridDisplay label="Modpacks" :instances="instances" />
  </div>
</template>
