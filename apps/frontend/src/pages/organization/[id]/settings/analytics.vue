<template>
  <div class="normal-page__content">
    <div class="universal-card">
      <h2>分析</h2>
      <p>
        此页面显示贵团队项目的分析数据。您可以查看所有资源的下载量、页面浏览量
      </p>
    </div>

    <ChartDisplay :projects="projects.map((x) => ({ title: x.name, ...x }))" />
  </div>
</template>

<script setup>
import ChartDisplay from "~/components/ui/charts/ChartDisplay.vue";

const { projects } = inject("organizationContext");
</script>

<style scoped lang="scss">
.markdown-body {
  margin-bottom: var(--gap-md);
}
</style>
