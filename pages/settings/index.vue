<template>
  <div>
    <section class="universal-card">
      <h2>Themes</h2>
      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">Color theme</span>
          <span class="label__description"
            >Change the global site color theme.</span
          >
        </label>
        <Multiselect
          id="theme-selector"
          v-model="$colorMode.preference"
          :options="['system', 'light', 'dark', 'oled']"
          :custom-label="
            (value) =>
              value === 'oled'
                ? 'OLED'
                : value.charAt(0).toUpperCase() + value.slice(1)
          "
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
        />
      </div>

      <div class="adjacent-input small">
        <label for="search-layout-toggle">
          <span class="label__title">Search sidebar on the right</span>
          <span class="label__description"
            >Enabling this will put the search page's filters sidebar on the
            right side.</span
          >
        </label>
        <input
          id="search-layout-toggle"
          v-model="searchLayout"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmeticSettings"
        />
      </div>
      <div class="adjacent-input small">
        <label for="project-layout-toggle">
          <span class="label__title">Project sidebar on the right</span>
          <span class="label__description"
            >Enabling this will put the project pages' info sidebars on the
            right side.</span
          >
        </label>
        <input
          id="project-layout-toggle"
          v-model="projectLayout"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmeticSettings"
        />
      </div>
    </section>
    <section class="universal-card">
      <h2>Project list display mode</h2>
      <div
        v-for="projectType in listTypes"
        :key="projectType.id + '-display-mode-selector'"
        class="adjacent-input small"
      >
        <label :for="projectType.id + '-search-display-mode'">
          <span class="label__title">{{ projectType.name }} display mode</span>
          <span class="label__description"
            >Change the display view for {{ projectType.display }}.</span
          >
        </label>
        <Multiselect
          :id="projectType + '-search-display-mode'"
          :value="searchDisplayMode[projectType.id]"
          :options="$tag.projectViewModes"
          :custom-label="$capitalizeString"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
          @input="(value) => setSearchDisplayMode(projectType.id, value)"
        />
      </div>
    </section>
    <section class="universal-card">
      <h2>Feature flags</h2>
      <div class="adjacent-input small">
        <label for="advanced-rendering">
          <span class="label__title">Advanced rendering</span>
          <span class="label__description"
            >Enables advanced rendering such as blur effects that may cause
            performance issues without hardware-accelerated rendering.</span
          >
        </label>
        <input
          id="advanced-rendering"
          v-model="advancedRendering"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmeticSettings"
        />
      </div>
      <div class="adjacent-input small">
        <label for="modpacks-alpha-notice">
          <span class="label__title">Modpacks alpha notice</span>
          <span class="label__description"
            >Shows a banner stating that modpacks are in alpha.</span
          >
        </label>
        <input
          id="modpacks-alpha-notice"
          v-model="modpacksAlphaNotice"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmeticSettings"
        />
      </div>
      <div class="adjacent-input small">
        <label for="external-links-new-tab">
          <span class="label__title">Open external links in new tab</span>
          <span class="label__description">
            Make links which go outside of Modrinth open in a new tab. No matter
            this setting, links on the same domain and in Markdown descriptions
            will open in the same tab, and links on ads and edit pages will open
            in a new tab.
          </span>
        </label>
        <input
          id="external-links-new-tab"
          v-model="externalLinksNewTab"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmeticSettings"
        />
      </div>
    </section>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'

export default {
  components: {
    Multiselect,
  },
  auth: false,
  data() {
    return {
      searchLayout: false,
      projectLayout: false,
      modpacksAlphaNotice: true,
      advancedRendering: true,
      externalLinksNewTab: true,
      searchDisplayMode: {
        mod: 'list',
        plugin: 'list',
        resourcepack: 'gallery',
        modpack: 'list',
        shader: 'gallery',
        user: 'list',
      },
    }
  },
  fetch() {
    this.searchLayout = this.$store.state.cosmetics.searchLayout
    this.projectLayout = this.$store.state.cosmetics.projectLayout
    this.modpacksAlphaNotice = this.$store.state.cosmetics.modpacksAlphaNotice
    this.advancedRendering = this.$store.state.cosmetics.advancedRendering
    this.externalLinksNewTab = this.$store.state.cosmetics.externalLinksNewTab
    this.searchDisplayMode = this.$store.state.cosmetics.searchDisplayMode
  },
  head: {
    title: 'Display settings - Modrinth',
  },
  computed: {
    listTypes() {
      const types = this.$tag.projectTypes.map((type) => {
        return {
          id: type.id,
          name: this.$formatProjectType(type.id) + ' search',
          display:
            'the ' +
            this.$formatProjectType(type.id).toLowerCase() +
            's search page',
        }
      })
      types.push({
        id: 'user',
        name: 'User page',
        display: 'user pages',
      })
      return types
    },
  },
  methods: {
    async saveCosmeticSettings() {
      await this.$store.dispatch('cosmetics/save', {
        searchLayout: this.searchLayout,
        projectLayout: this.projectLayout,
        modpacksAlphaNotice: this.modpacksAlphaNotice,
        advancedRendering: this.advancedRendering,
        externalLinksNewTab: this.externalLinksNewTab,
        searchDisplayMode: this.searchDisplayMode,
        $cookies: this.$cookies,
      })
    },
    async setSearchDisplayMode(projectType, value) {
      await this.$store.dispatch('cosmetics/saveSearchDisplayMode', {
        projectType,
        mode: value,
        $cookies: this.$cookies,
      })
      this.searchDisplayMode = this.$store.state.cosmetics.searchDisplayMode
    },
    changeTheme() {
      const shift = event.shiftKey
      switch (this.$colorMode.preference) {
        case 'dark':
          this.$colorMode.preference = shift ? 'light' : 'oled'
          break
        case 'oled':
          this.$colorMode.preference = shift ? 'dark' : 'light'
          break
        default:
          this.$colorMode.preference = shift ? 'oled' : 'dark'
      }
    },
  },
}
</script>
<style lang="scss" scoped></style>
