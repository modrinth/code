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
      <Button
        class="no-wrap clear-filters"
        :disabled="!filterLoader && !filterVersions"
        :action="clearFilters"
      >
        <ClearIcon />
        Clear filters
      </Button>
    </div>
  </Card>
  <Card class="mod-card">
    <div class="table">
      <div class="table-row table-head">
        <div class="table-cell table-text download-cell" />
        <div class="name-cell table-cell table-text">Name</div>
        <div class="table-cell table-text">Supports</div>
        <div class="table-cell table-text">Stats</div>
      </div>
      <div
        v-for="version in versions"
        :key="version.id"
        class="table-row selectable"
        @click="$router.push(`/project/${$route.params.id}/version/${version.id}`)"
      >
        <div class="table-cell table-text">
          <Button
            color="primary"
            icon-only
            :disabled="installed"
            @click.stop="() => install(version.id)"
          >
            <DownloadIcon v-if="!installed" />
            <CheckIcon v-else />
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
      </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, CheckIcon, ClearIcon, Badge, DownloadIcon, formatNumber } from 'omorphia'
import Multiselect from 'vue-multiselect'
import { releaseColor } from '@/helpers/utils'
import { ref } from 'vue'

let filterVersions = ref(null)
let filterLoader = ref(null)

const clearFilters = () => {
  filterVersions.value = null
  filterLoader.value = null
}

defineProps({
  versions: {
    type: Array,
    required: true,
  },
  install: {
    type: Function,
    required: true,
  },
  installed: {
    type: Boolean,
    required: true,
  },
})
</script>

<style scoped lang="scss">
.filter-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.table-row {
  grid-template-columns: min-content 1fr 1fr 1.5fr;
}

.manage {
  display: flex;
  gap: 0.5rem;
  flex-grow: 1;

  .multiselect {
    flex-grow: 1;
  }
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
