<script setup>

import ProgressBar from "@/components/ui/ProgressBar.vue";
import {Button, Card} from "omorphia";

defineProps({
  progress: {
    type: Number,
    default: 0,
  },
  title: {
    type: String,
    default: "Tutorial",
  },
  description: {
    type: String,
    default: "This is a tutorial",
  },
  progressFunction: {
    type: Function,
    default: () => {},
  },
})
</script>

<template>
  <Card class="tutorial-card">
    <h3 class="tutorial-title">
      {{title}}
    </h3>
    <div class="tutorial-body">
      {{description}}
    </div>
    <div class="tutorial-footer">
      {{progress}}/9
      <ProgressBar :progress="(progress/9) * 100"/>
      <Button color="primary" :action="progressFunction">
        {{progress === 9 ? "Finish" : "Next"}}
      </Button>
    </div>
  </Card>
</template>

<style scoped lang="scss">
.tutorial-card {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
  border: 1px solid var(--color-button-bg);
  width: 20rem;
}

.tutorial-title {
  margin: 0;
}

.tutorial-footer {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);
}
</style>
