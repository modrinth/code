<template>
  <div>
    <div class="universal-card">
      <h2>分析</h2>

      <p>
        该页面显示您<strong>{{ project.title }}</strong
        >的分析数据 . 您可以查看该资源的到下载次数, 浏览量等
      </p>
    </div>

    <ChartDisplay :projects="[props.project]" />
  </div>
</template>

<script setup>
import ChartDisplay from "~/components/ui/charts/ChartDisplay.vue";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
});
</script>

<style scoped lang="scss">
.markdown-body {
  margin-bottom: var(--gap-md);
}
</style>
