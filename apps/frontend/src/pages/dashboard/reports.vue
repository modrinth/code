<template>
  <div>
    <section class="universal-card">
      <h2 class="text-2xl">报告</h2>
      <ReportsList :auth="auth" />
    </section>
  </div>
</template>
<script setup>
import ReportsList from "~/components/ui/report/ReportsList.vue";

const auth = await useAuth();
useHead({
  title: "Active reports - BBSMC",
});
</script>
