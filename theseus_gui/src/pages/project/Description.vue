<template>
  <Card>
    <div class="markdown-body" v-html="renderHighlightedString(body)"/>
  </Card>
</template>

<script setup>
import {Card, renderHighlightedString} from 'omorphia'
import { useProjectStore } from '@/store/project';
const body = useProjectStore().project.body;
</script>

<script>
export default {
  name: "Description"
}
</script>

<style scoped lang="scss">
</style>
