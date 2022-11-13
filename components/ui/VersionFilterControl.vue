<template>
  <div
    v-if="getValidLoaders().length > 1 || getValidVersions().length > 1"
    class="card search-controls"
  >
    <Multiselect
      v-if="getValidLoaders().length > 1"
      v-model="selectedLoaders"
      :options="getValidLoaders()"
      :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
      :multiple="true"
      :searchable="false"
      :show-no-results="false"
      :close-on-select="true"
      :clear-search-on-select="false"
      :show-labels="false"
      :allow-empty="true"
      :disabled="getValidLoaders().length === 1"
      placeholder="Filter loader..."
      @input="updateVersionFilters()"
    ></Multiselect>
    <Multiselect
      v-if="getValidVersions().length > 1"
      v-model="selectedGameVersions"
      :options="
        showSnapshots
          ? getValidVersions().map((x) => x.version)
          : getValidVersions()
              .filter((it) => it.version_type === 'release')
              .map((x) => x.version)
      "
      :multiple="true"
      :searchable="true"
      :show-no-results="false"
      :close-on-select="false"
      :show-labels="false"
      :hide-selected="true"
      :selectable="() => selectedGameVersions.length <= 6"
      placeholder="Filter versions..."
      @input="updateVersionFilters()"
    ></Multiselect>
    <Checkbox
      v-if="
        getValidVersions().length > 1 &&
        getValidVersions().some((v) => v.version_type !== 'release')
      "
      v-model="showSnapshots"
      label="Include snapshots"
      description="Include snapshots"
      :border="false"
    />
    <button
      title="Clear filters"
      :disabled="
        selectedLoaders.length === 0 && selectedGameVersions.length === 0
      "
      class="iconified-button"
      @click="
        selectedLoaders = []
        selectedGameVersions = []
        updateVersionFilters()
      "
    >
      <ClearIcon />
      Clear filters
    </button>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import Checkbox from '~/components/ui/Checkbox'
import ClearIcon from '~/assets/images/utils/clear.svg?inline'
export default {
  name: 'VersionFilterControl',
  components: {
    Multiselect,
    Checkbox,
    ClearIcon,
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
        this.cachedValidVersions = this.$tag.gameVersions.filter((gameVer) =>
          this.versions.some((projVer) =>
            projVer.game_versions.includes(gameVer.version)
          )
        )
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

<style lang="scss" scoped>
.search-controls {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-card-md);
  align-items: center;
  flex-wrap: wrap;
  .multiselect {
    flex: 1;
  }
  .checkbox-outer {
    min-width: fit-content;
  }
}
.circle-button {
  display: flex;
  max-width: 2rem;
  padding: 0.5rem;
  background-color: var(--color-button-bg);
  border-radius: var(--size-rounded-max);
  box-shadow: inset 0px -1px 1px rgba(17, 24, 39, 0.1);
  &:hover,
  &:focus-visible {
    background-color: var(--color-button-bg-hover);
    color: var(--color-button-text-hover);
  }
  &:active {
    background-color: var(--color-button-bg-active);
    color: var(--color-button-text-active);
  }
  svg {
    height: 1rem;
    width: 1rem;
  }
}
</style>
