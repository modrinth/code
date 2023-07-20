<template>
  <div>
    <section class="universal-card">
      <h2>Statistics</h2>
      <div class="grid-display">
        <div class="grid-display__item">
          <div class="label">Projects</div>
          <div class="value">
            {{ formatNumber(stats.projects) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Versions</div>
          <div class="value">
            {{ formatNumber(stats.versions) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Files</div>
          <div class="value">
            {{ formatNumber(stats.files) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Authors</div>
          <div class="value">
            {{ formatNumber(stats.authors) }}
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
<script setup>
import { formatNumber } from '~/plugins/shorthands.js'

useHead({
  title: 'Staff overview - Modrinth',
})

const { data: stats } = await useAsyncData('statistics', () => useBaseFetch('statistics'))
</script>
