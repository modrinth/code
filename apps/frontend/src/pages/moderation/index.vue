<template>
  <div>
    <section class="universal-card">
      <h2>Statistics</h2>
      <div class="grid-display">
        <div class="grid-display__item">
          <div class="label">Projects</div>
          <div class="value">
            {{ formatNumber(stats.projects, false) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Versions</div>
          <div class="value">
            {{ formatNumber(stats.versions, false) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Files</div>
          <div class="value">
            {{ formatNumber(stats.files, false) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Authors</div>
          <div class="value">
            {{ formatNumber(stats.authors, false) }}
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
<script setup>
import { formatNumber } from "~/plugins/shorthands.js";

useHead({
  title: "Staff overview - Modrinth",
});

const { data: stats } = await useAsyncData("statistics", () => useBaseFetch("statistics"));
</script>
