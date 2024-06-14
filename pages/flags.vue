<script setup lang="ts">
import {
  type FeatureFlag,
  DEFAULT_FEATURE_FLAGS,
  saveFeatureFlags,
} from '~/composables/featureFlags.ts'

const flags = shallowReactive(useFeatureFlags().value)
</script>

<template>
  <div class="page">
    <h1>Feature flags</h1>
    <div class="flags">
      <div
        v-for="flag in Object.keys(flags) as FeatureFlag[]"
        :key="`flag-${flag}`"
        class="adjacent-input small card"
      >
        <label :for="`toggle-${flag}`">
          <span class="label__title">
            {{ flag.replaceAll('_', ' ') }}
          </span>
          <span class="label__description">
            <p>
              Default:
              <span
                :style="`color:var(--color-${
                  DEFAULT_FEATURE_FLAGS[flag] === false ? 'red' : 'green'
                })`"
                >{{ DEFAULT_FEATURE_FLAGS[flag] }}</span
              >
            </p>
          </span>
        </label>
        <input
          :id="`toggle-${flag}`"
          v-model="flags[flag]"
          class="switch stylized-toggle"
          type="checkbox"
          @change="() => saveFeatureFlags()"
        />
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.page {
  width: calc(100% - 2 * var(--spacing-card-md));
  max-width: 800px;
  margin-inline: auto;
  box-sizing: border-box;
  margin-block: var(--spacing-card-md);
}

.flags {
}

.label__title {
  text-transform: capitalize;
}

.label__description p {
  margin: 0;
}
</style>
