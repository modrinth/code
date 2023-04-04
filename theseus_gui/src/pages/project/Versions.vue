<template>
  <Card>
    <div class="filter-header">
      <div class="manage">
        <DropdownSelect id="filterLoader" placeholder="Filter loader..." :options="versions.flatMap(value => value.loaders).filter((value, index, self) => self.indexOf(value) === index)" class="select no-wrap"/>
        <DropdownSelect id="filterVersions" placeholder="Filter versions..."  :options="versions.flatMap(value => value.game_versions).filter((value, index, self) => self.indexOf(value) === index)" class="select no-wrap"/>
      </div>
      <div class="manage">
        <Checkbox id="filterCompatible" label="Only show compatible versions" />
        <Button class="no-wrap">
          <CheckCircleIcon />
          Clear Filters
        </Button>
      </div>
    </div>
  </Card>
  <Card class="mod-card">
    <div class="table-container">
      <div class="table-row table-head">
        <div class="table-cell table-text download-cell" />
        <div class="name-cell table-cell table-text">Name</div>
        <div class="table-cell table-text">Supports</div>
        <div class="table-cell table-text">Stats</div>
      </div>
      <div v-for="(version) in versions" :key="version.id" class="table-row" >
        <div class="table-cell table-text">
          <Button color="primary" icon-only>
            <DownloadIcon />
          </Button>
        </div>
        <div class="name-cell table-cell table-text">
          <router-link :to="`/project/${$route.params.id}/version/${version.id}`" class="version-link">
            {{ version.name.charAt(0).toUpperCase() + version.name.slice(1) }}
            <div class="version-badge">
              <div class="channel-indicator">
                <Badge :color="releaseColor(version.version_type)" :type="version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)" />
              </div>
              <div>
                {{ version.version_number }}
              </div>
            </div>
          </router-link>
        </div>
        <div class="table-cell table-text stacked-text">
          <span>
            {{ version.loaders.map(str => str.charAt(0).toUpperCase() + str.slice(1)).join(', ') }}
          </span>
          <span>
            {{ version.game_versions.join(', ') }}
          </span>
        </div>
        <div class="table-cell table-text stacked-text">
          <div>
            <span>
              Published on
            </span>
            <strong>
              {{ new Date(version.date_published).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' }) }}
            </strong>
          </div>
          <div>
            <strong>
              {{ formatNumber(version.downloads) }}
            </strong>
            <span>
              Downloads
            </span>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, DropdownSelect, CheckCircleIcon, Badge, DownloadIcon, Checkbox, formatNumber } from 'omorphia'
import { releaseColor } from '@/helpers/utils'
</script>

<script>
export default {
  name: "Versions",
  data() {
    return {
      versions: [],
    };
  },
  async mounted() {
    const response = await fetch(`https://api.modrinth.com/v2/project/${this.$route.params.id}/version`);
    this.versions = await response.json();
  },
}
</script>

<style scoped lang="scss">
.filter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.table-container {
  display: grid;
  grid-template-rows: repeat(auto-fill, auto);
  width: 100%;
  border-radius: var(--radius-md);
  overflow: hidden;
}

.table-row {
  display: grid;
  grid-template-columns: min-content 1fr 1fr 1.5fr;
}

.table-head {
  .table-cell {
    background-color: var(--color-accent-contrast);
  }
}

.table-cell {
  padding: 1rem;
  height: 100%;
  align-items: center;
  display: flex;
}

.name-cell {
  padding-left: 0;
}

.table-text {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: fade;
}

.manage {
  display: flex;
  gap: 0.5rem;
  flex-grow: 1;
  margin-right: 0.5rem;
}

.mod-text {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: var(--color-contrast);
}

.table-row:nth-child(even) .table-cell {
  background-color: var(--color-bg);
}

.card-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.select {
  width: 100% !important;
  max-width: 20rem;
}

.version-link {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;

  .version-badge {
    display: flex;
    flex-wrap: wrap;

    .channel-indicator {
      margin-right: 0.5rem;
    }
  }
}

.stacked-text {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  align-items: flex-start;
}

.download-cell {
  width: 4rem;
  padding: 1rem;
}
</style>
