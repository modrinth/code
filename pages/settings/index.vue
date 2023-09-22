<template>
  <div>
    <section class="universal-card">
      <h2>Themes</h2>
      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">Color theme</span>
          <span class="label__description">Change the global site color theme.</span>
        </label>
        <div>
          <Multiselect
            id="theme-selector"
            v-model="$colorMode.preference"
            :options="['system', 'light', 'dark', 'oled']"
            :custom-label="
              (value) =>
                value === 'oled' ? 'OLED' : value.charAt(0).toUpperCase() + value.slice(1)
            "
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
            @update:model-value="(value) => updateTheme(value, true)"
          />
        </div>
      </div>

      <div class="adjacent-input small">
        <label for="search-layout-toggle">
          <span class="label__title">Search sidebar on the right</span>
          <span class="label__description"
            >Enabling this will put the search page's filters sidebar on the right side.</span
          >
        </label>
        <input
          id="search-layout-toggle"
          v-model="cosmetics.searchLayout"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmetics"
        />
      </div>
      <div class="adjacent-input small">
        <label for="project-layout-toggle">
          <span class="label__title">Project sidebar on the right</span>
          <span class="label__description"
            >Enabling this will put the project pages' info sidebars on the right side.</span
          >
        </label>
        <input
          id="project-layout-toggle"
          v-model="cosmetics.projectLayout"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmetics"
        />
      </div>
    </section>
    <section class="universal-card">
      <h2>Project list display mode</h2>
      <div
        v-for="projectType in listTypes"
        :key="projectType.id + '-display-mode-selector'"
        class="adjacent-input"
      >
        <label :for="projectType.id + '-search-display-mode'">
          <span class="label__title">{{ projectType.name }} display mode</span>
          <span class="label__description"
            >Change the display view for {{ projectType.display }}.</span
          >
        </label>
        <Multiselect
          :id="projectType + '-search-display-mode'"
          v-model="cosmetics.searchDisplayMode[projectType.id]"
          :options="tags.projectViewModes"
          :custom-label="$capitalizeString"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
          @update:model-value="saveCosmetics"
        />
      </div>
    </section>
    <section class="universal-card">
      <h2>Feature flags</h2>
      <div class="adjacent-input small">
        <label for="advanced-rendering">
          <span class="label__title">Advanced rendering</span>
          <span class="label__description"
            >Enables advanced rendering such as blur effects that may cause performance issues
            without hardware-accelerated rendering.</span
          >
        </label>
        <input
          id="advanced-rendering"
          v-model="cosmetics.advancedRendering"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmetics"
        />
      </div>
      <div class="adjacent-input small">
        <label for="external-links-new-tab">
          <span class="label__title">Open external links in new tab</span>
          <span class="label__description">
            Make links which go outside of Modrinth open in a new tab. No matter this setting, links
            on the same domain and in Markdown descriptions will open in the same tab, and links on
            ads and edit pages will open in a new tab.
          </span>
        </label>
        <input
          id="external-links-new-tab"
          v-model="cosmetics.externalLinksNewTab"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmetics"
        />
      </div>
      <div class="adjacent-input small">
        <label for="modrinth-app-promos">
          <span class="label__title">Hide Modrinth App promotions</span>
          <span class="label__description">
            Hides the "Get Modrinth App" buttons from primary navigation. The Modrinth App page can
            still be found on the landing page or in the footer.
          </span>
        </label>
        <input
          id="modrinth-app-promos"
          v-model="cosmetics.hideModrinthAppPromos"
          class="switch stylized-toggle"
          type="checkbox"
          @change="saveCosmetics"
        />
      </div>
    </section>
  </div>
</template>

<script>
import { Multiselect } from 'vue-multiselect'

export default defineNuxtComponent({
  components: {
    Multiselect,
  },
  setup() {
    const cosmetics = useCosmetics()
    const tags = useTags()

    return { cosmetics, tags }
  },
  data() {
    return {
      searchDisplayMode: this.cosmetics.searchDisplayMode,
    }
  },
  head: {
    title: 'Display settings - Modrinth',
  },
  computed: {
    listTypes() {
      const types = this.tags.projectTypes.map((type) => {
        return {
          id: type.id,
          name: this.$formatProjectType(type.id) + ' search',
          display: 'the ' + this.$formatProjectType(type.id).toLowerCase() + 's search page',
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
})
</script>
