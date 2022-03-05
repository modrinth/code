<template>
  <div class="card search-controls">
    <Multiselect
      v-model="selectedLoaders"
      :options="getValidLoaders()"
      :multiple="true"
      :searchable="true"
      :show-no-results="false"
      :close-on-select="false"
      :clear-search-on-select="false"
      :show-labels="false"
      :selectable="() => selectedLoaders.length <= 6"
      placeholder="Filter loaders..."
      @input="updateVersionFilters()"
    ></Multiselect>
    <Multiselect
      v-model="selectedGameVersions"
      :options="getValidVersions()"
      :multiple="true"
      :searchable="true"
      :show-no-results="false"
      :close-on-select="false"
      :clear-search-on-select="false"
      :show-labels="false"
      :selectable="() => selectedGameVersions.length <= 6"
      placeholder="Filter versions..."
      @input="updateVersionFilters()"
    ></Multiselect>
    <Checkbox
      v-model="showSnapshots"
      label="Include snapshots"
      description="Include snapshots"
      style="margin-bottom: 0.5rem"
      :border="false"
    />
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import Checkbox from '~/components/ui/Checkbox'

export default {
  name: 'VersionFilterControl',
  components: {
    Multiselect,
    Checkbox,
  },
  props: {
    versions: {
      type: Array,
      required: true,
    },
  },
  data() {
    return {
      query: '',
      showSnapshots: false,
      cachedValidVersions: null,
      cachedValidLoaders: null,
      selectedGameVersions: [],
      selectedLoaders: [],
    }
  },
  methods: {
    getValidVersions() {
      if (!this.cachedValidVersions) {
        const temp = new Set()
        for (const version of this.versions) {
          version.game_versions.forEach((v) => {
            temp.add(v)
          })
        }
        this.cachedValidVersions = Array.from(temp)
        this.cachedValidVersions.sort().reverse()
      }
      return this.cachedValidVersions
    },
    getValidLoaders() {
      if (!this.cachedValidLoaders) {
        const temp = new Set()
        for (const version of this.versions) {
          version.loaders.forEach((v) => {
            temp.add(v)
          })
        }
        this.cachedValidLoaders = Array.from(temp)
        this.cachedValidLoaders.sort()
      }
      return this.cachedValidLoaders
    },
    updateVersionFilters() {
      const temp = this.versions.filter(
        (projectVersion) =>
          (this.selectedGameVersions.length === 0 ||
            this.selectedGameVersions.some((gameVersion) =>
              projectVersion.game_versions.includes(gameVersion)
            )) &&
          (this.selectedLoaders.length === 0 ||
            this.selectedLoaders.some((loader) =>
              projectVersion.loaders.includes(loader)
            ))
      )
      this.$emit('updateVersions', temp)
    },
  },
}
</script>

<style lang="scss">
.search-controls {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-card-md);
  align-items: center;
  flex-wrap: wrap;

  .multiselect {
    flex-grow: 1;
    max-height: unset;
    width: fit-content;

    input::placeholder {
      color: var(--color-text);
    }
  }

  .checkbox-outer {
    margin-bottom: 0 !important;
  }
}
</style>
