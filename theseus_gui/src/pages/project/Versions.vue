<template>
  <Card>
    <div class="filter-header">
      <div class="manage">
        <multiselect
          v-model="filterVersions"
          :options="
            versions
              .flatMap((value) => value.loaders)
              .filter((value, index, self) => self.indexOf(value) === index)
          "
          :multiple="true"
          :searchable="true"
          :show-no-results="false"
          :close-on-select="false"
          :clear-search-on-select="false"
          :show-labels="false"
          :selectable="() => versions.length <= 6"
          placeholder="Filter loader..."
        />
        <multiselect
          v-model="filterLoader"
          :options="
            versions
              .flatMap((value) => value.game_versions)
              .filter((value, index, self) => self.indexOf(value) === index)
          "
          :multiple="true"
          :searchable="true"
          :show-no-results="false"
          :close-on-select="false"
          :clear-search-on-select="false"
          :show-labels="false"
          :selectable="() => versions.length <= 6"
          placeholder="Filter versions..."
        />
      </div>
      <Checkbox
        v-model="filterCompatible"
        label="Only show compatible versions"
        class="filter-checkbox"
      />
      <Button
        class="no-wrap clear-filters"
        :disabled="!filterLoader && !filterVersions && !filterCompatible"
        :action="clearFilters"
      >
        <CheckCircleIcon />
        Clear Filters
      </Button>
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
      <router-link
        v-for="version in versions"
        :key="version.id"
        class="button-base table-row"
        :to="`/project/${$route.params.id}/version/${version.id}`"
      >
        <div class="table-cell table-text">
          <Button color="primary" icon-only>
            <DownloadIcon />
          </Button>
        </div>
        <div class="name-cell table-cell table-text">
          <div class="version-link">
            {{ version.name.charAt(0).toUpperCase() + version.name.slice(1) }}
            <div class="version-badge">
              <div class="channel-indicator">
                <Badge
                  :color="releaseColor(version.version_type)"
                  :type="
                    version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)
                  "
                />
              </div>
              <div>
                {{ version.version_number }}
              </div>
            </div>
          </div>
        </div>
        <div class="table-cell table-text stacked-text">
          <span>
            {{
              version.loaders.map((str) => str.charAt(0).toUpperCase() + str.slice(1)).join(', ')
            }}
          </span>
          <span>
            {{ version.game_versions.join(', ') }}
          </span>
        </div>
        <div class="table-cell table-text stacked-text">
          <div>
            <span> Published on </span>
            <strong>
              {{
                new Date(version.date_published).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'short',
                  day: 'numeric',
                })
              }}
            </strong>
          </div>
          <div>
            <strong>
              {{ formatNumber(version.downloads) }}
            </strong>
            <span> Downloads </span>
          </div>
        </div>
      </router-link>
    </div>
  </Card>
</template>

<script setup>
import {
  Card,
  Button,
  CheckCircleIcon,
  Badge,
  DownloadIcon,
  Checkbox,
  formatNumber,
} from 'omorphia'
import Multiselect from 'vue-multiselect'
import { releaseColor } from '@/helpers/utils'
</script>

<script>
export default {
  name: 'Versions',
  data() {
    return {
      versions: [],
      filterVersions: null,
      filterLoader: null,
      filterCompatible: false,
    }
  },
  async mounted() {
    const response = await fetch(
      `https://api.modrinth.com/v2/project/${this.$route.params.id}/version`
    )
    this.versions = await response.json()
  },
  methods: {
    clearFilters() {
      this.filterVersions = null
      this.filterLoader = null
      this.filterCompatible = false
    },
  },
}
</script>

<style scoped lang="scss">
.filter-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
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
  background-color: var(--color-raised-bg);
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

  .multiselect {
    flex-grow: 1;
  }
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

.filter-checkbox {
  :deep(.checkbox) {
    border: none;
  }
}
</style>
