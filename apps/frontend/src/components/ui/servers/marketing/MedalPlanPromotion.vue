<template>
  <div
    id="medal"
    class="medal-promotion flex w-full flex-row justify-between rounded-xl p-6 shadow-xl"
  >
    <div class="overlay"></div>
    <MedalPromoBackground class="background-pattern shadow-xl" />
    <div class="z-10 flex flex-col gap-2">
      <div class="flex items-center gap-2 text-2xl font-semibold text-contrast">
        <ClockIcon class="clock-glow size-6 text-orange" /><span>
          Try a free
          <span class="text-orange">3GB server</span> for 5 days powered by
          <span class="text-orange">Medal</span>
        </span>
      </div>
      <div class="flex items-center">
        <span class="text-sm text-secondary"
          >Limited-time offer. No credit card required. Available for US servers.</span
        >
      </div>
    </div>
    <ButtonStyled color="orange" type="outlined" size="large">
      <nuxt-link to="https://medal.tv/" class="z-10 my-auto">Learn more <ExternalIcon /></nuxt-link>
    </ButtonStyled>
  </div>
</template>

<script lang="ts" setup>
import { ClockIcon, ExternalIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import MedalPromoBackground from "~/assets/images/illustrations/medal_promo_background.svg?component";
</script>

<style scoped lang="scss">
.medal-promotion {
  position: relative;
  border: 1px solid var(--medal-promotion-bg-orange);
  overflow: hidden;
}

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: var(--medal-promotion-bg-gradient);
  z-index: 1;
  border-radius: inherit;
}

.background-pattern {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  background-color: var(--medal-promotion-bg);
  border-radius: inherit;
  color: var(--color-orange);
}

.clock-glow {
  filter: drop-shadow(0 0 72px var(--color-orange)) drop-shadow(0 0 36px var(--color-orange))
    drop-shadow(0 0 18px var(--color-orange));
}

.text-orange {
  color: var(--medal-promotion-text-orange);
}
</style>
