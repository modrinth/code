<template>
  <div class="GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD">
    <div class="GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-0">
      <div class="GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-1">
        <div class="GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-2">
          <a
            :href="`https://bisecthosting.com/modrinth${queryParam}`"
            rel="noopener nofollow sponsored"
            :target="target"
          >
            <BisectIcon class="GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-3" />
            <span>
              <span> Host your Minecraft server on </span>
              <strong>BisectHosting</strong>
              <span> - get 25% off your first month with code <strong>MODRINTH</strong>. </span>
            </span>
          </a>
        </div>
      </div>
      <div class="GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-4">
        <a rel="noopener sponsored" :target="target" href="https://adrinth.com">
          Ads via Adrinth
        </a>
      </div>
    </div>
  </div>
</template>
<script setup>
import { computed } from 'vue'
import BisectIcon from '@/assets/external/bh.svg'
const props = defineProps({
  external: {
    type: Boolean,
    default: true,
  },
  queryParam: {
    type: String,
    default: '',
  },
})
const target = computed(() => (props.external ? '_blank' : '_self'))
</script>

<style lang="scss">
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD {
  position: relative;
  margin-bottom: var(--gap-md);
  background: var(--color-ad);
  border: 3px solid var(--color-ad-raised);
  border-radius: var(--radius-lg);
}
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-0 {
  font-size: 14px;
  line-height: 1.3em;
}
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-1 {
  color: var(--color-base);
  padding: 1em;
  text-align: left;
}
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-2 a {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-base);

  &:hover {
    text-decoration: none;
  }
}
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-2 a b,
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-2 a strong {
  color: #088cdb;
}
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-3 {
  padding-top: 1px;
  height: 1.2rem;
  width: auto;
}
.GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-4 a {
  position: absolute;
  bottom: -2px;
  right: -2px;
  text-align: center;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.8em;
  color: var(--color-base);
  background: var(--color-ad-raised);
  letter-spacing: 0.1ch;
  margin: 0;
  padding: 2px 10px;
  border-top-left-radius: var(--radius-lg);
  border-bottom-right-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  gap: 0.5rem;

  &:hover {
    text-decoration: none;
  }
}
@media screen and (max-width: 800px) {
  .GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-2 {
    margin-bottom: 0.5rem;
  }
  .GBBNWLJVGRHFLYVGSZKSSKNTHFYXHMBD-2 a {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
