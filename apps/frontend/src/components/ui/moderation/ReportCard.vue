<template>
  <div class="universal-card">
    <h3>Reported for {{ report. }}</h3>
  </div>
</template>

<script setup lang="ts">
import type { ExtendedReport } from "~/pages/moderation/reports.vue";

defineProps<{
  report: ExtendedReport;
}>();
</script>
