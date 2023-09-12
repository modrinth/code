<script setup>
import { Card, Slider, DropdownSelect, Toggle } from 'omorphia'
import JavaSelector from '@/components/ui/JavaSelector.vue'

const pageOptions = ['Home', 'Library']
</script>

<template>
  <div class="settings-page">
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Display</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title">Color theme</span>
          <span class="label__description">Change the global launcher color theme.</span>
        </label>
        <DropdownSelect
          id="theme"
          name="Theme dropdown"
          :options="['Dark']"
          :disabled="true"
          :default-value="'dark'"
          class="theme-dropdown disable-children"
        />
      </div>
      <div class="adjacent-input">
        <label for="collapsed-nav">
          <span class="label__title">Collapsed navigation mode</span>
          <span class="label__description"
            >Change the style of the side navigation bar to a compact version.</span
          >
        </label>
        <Toggle id="collapsed-nav" :checked="false" :disabled="true" />
      </div>
      <div class="adjacent-input">
        <label for="advanced-rendering">
          <span class="label__title">Advanced rendering</span>
          <span class="label__description">
            Enables advanced rendering such as blur effects that may cause performance issues
            without hardware-accelerated rendering.
          </span>
        </label>
        <Toggle id="advanced-rendering" :checked="true" :disabled="true" />
      </div>
      <div class="adjacent-input">
        <label for="minimize-launcher">
          <span class="label__title">Minimize launcher</span>
          <span class="label__description"
            >Minimize the launcher when a Minecraft process starts.</span
          >
        </label>
        <Toggle id="minimize-launcher" :checked="false" :disabled="true" />
      </div>
      <div class="opening-page">
        <label for="opening-page">
          <span class="label__title">Default landing page</span>
          <span class="label__description">Change the page to which the launcher opens on.</span>
        </label>
        <DropdownSelect
          id="opening-page"
          name="Opening page dropdown"
          :options="pageOptions"
          default-value="Home"
          class="opening-page"
          :disabled="true"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Resource management</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="max-downloads">
          <span class="label__title">Maximum concurrent downloads</span>
          <span class="label__description"
            >The maximum amount of files the launcher can download at the same time. Set this to a
            lower value if you have a poor internet connection.</span
          >
        </label>
        <Slider id="max-downloads" :min="1" :max="10" :step="1" :disabled="true" />
      </div>

      <div class="adjacent-input">
        <label for="max-writes">
          <span class="label__title">Maximum concurrent writes</span>
          <span class="label__description"
            >The maximum amount of files the launcher can write to the disk at once. Set this to a
            lower value if you are frequently getting I/O errors.</span
          >
        </label>
        <Slider id="max-writes" :min="1" :max="50" :step="1" :disabled="true" />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Privacy</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="opt-out-analytics">
          <span class="label__title">Disable analytics</span>
          <span class="label__description">
            Modrinth collects anonymized analytics and usage data to improve our user experience and
            customize your experience. Opting out will disable this data collection.
          </span>
        </label>
        <Toggle id="opt-out-analytics" :disabled="true" />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Java settings</span>
        </h3>
      </div>
      <label for="java-17">
        <span class="label__title">Java 17 location</span>
      </label>
      <JavaSelector id="java-17" :version="17" model-value="" :disabled="true" />
      <label for="java-8">
        <span class="label__title">Java 8 location</span>
      </label>
      <JavaSelector id="java-8" :version="8" model-value="" :disabled="true" />
      <hr class="card-divider" />
      <label for="java-args">
        <span class="label__title">Java arguments</span>
      </label>
      <input
        id="java-args"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter java arguments..."
        :disabled="true"
      />
      <label for="env-vars">
        <span class="label__title">Environmental variables</span>
      </label>
      <input
        id="env-vars"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter environmental variables..."
        :disabled="true"
      />
      <hr class="card-divider" />
      <div class="adjacent-input">
        <label for="max-memory">
          <span class="label__title">Java memory</span>
          <span class="label__description">
            The memory allocated to each instance when it is ran.
          </span>
        </label>
        <Slider id="max-memory" :min="256" :max="10256" :step="1" unit="mb" :disabled="true" />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Hooks</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="pre-launch">
          <span class="label__title">Pre launch</span>
          <span class="label__description"> Ran before the instance is launched. </span>
        </label>
        <input
          id="pre-launch"
          autocomplete="off"
          type="text"
          placeholder="Enter pre-launch command..."
          :disabled="true"
        />
      </div>
      <div class="adjacent-input">
        <label for="wrapper">
          <span class="label__title">Wrapper</span>
          <span class="label__description"> Wrapper command for launching Minecraft. </span>
        </label>
        <input
          id="wrapper"
          autocomplete="off"
          type="text"
          placeholder="Enter wrapper command..."
          :disabled="true"
        />
      </div>
      <div class="adjacent-input">
        <label for="post-exit">
          <span class="label__title">Post exit</span>
          <span class="label__description"> Ran after the game closes. </span>
        </label>
        <input
          id="post-exit"
          autocomplete="off"
          type="text"
          placeholder="Enter post-exit command..."
          :disabled="true"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Window size</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="width">
          <span class="label__title">Width</span>
          <span class="label__description"> The width of the game window when launched. </span>
        </label>
        <input
          id="width"
          autocomplete="off"
          type="number"
          placeholder="Enter width..."
          :disabled="true"
        />
      </div>
      <div class="adjacent-input">
        <label for="height">
          <span class="label__title">Height</span>
          <span class="label__description"> The height of the game window when launched. </span>
        </label>
        <input
          id="height"
          autocomplete="off"
          type="number"
          class="input"
          placeholder="Enter height..."
          :disabled="true"
        />
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.settings-page {
  margin: 1rem;
}

.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.theme-dropdown {
  text-transform: capitalize;
}

.card-divider {
  margin: 1rem 0;
}

.disable-children * {
  pointer-events: none;
}
</style>
