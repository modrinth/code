<script setup lang="ts">
import { RightArrowIcon, ModrinthIcon, XIcon } from '@modrinth/assets'
import ButtonStyled from '../base/ButtonStyled.vue'
import AutoLink from '../base/AutoLink.vue'

const emit = defineEmits<{
  (e: 'close'): void
}>()

withDefaults(
  defineProps<{
    link: string
    closable?: boolean
  }>(),
  {
    closable: true,
  },
)
</script>
<template>
  <div
    class="brand-gradient-bg card-shadow bg-bg relative p-4 border-[1px] border-solid border-brand rounded-2xl grid grid-cols-[1fr_auto] overflow-hidden"
  >
    <ModrinthIcon
      class="absolute -top-12 -right-12 size-48 text-brand-highlight opacity-25"
      fill="none"
      stroke="var(--color-brand)"
      stroke-width="4"
    />
    <div class="flex flex-col gap-2">
      <span class="text-lg leading-tight font-extrabold text-contrast"
        >Want to play with <br />
        <span class="text-brand">your friends?</span></span
      >
      <span class="text-sm font-medium">Create a server with Modrinth in just a few clicks.</span>
    </div>
    <div class="flex flex-col items-end justify-end z-10">
      <ButtonStyled color="brand">
        <AutoLink :to="link"> View plans <RightArrowIcon /> </AutoLink>
      </ButtonStyled>
    </div>
    <div class="absolute top-2 right-2 z-10">
      <ButtonStyled v-if="closable" size="small" circular>
        <button v-tooltip="`Don't show again`" @click="emit('close')">
          <XIcon aria-hidden="true" />
        </button>
      </ButtonStyled>
    </div>
  </div>
</template>
<style scoped>
.brand-gradient-bg {
  background-image: linear-gradient(
    to top right,
    var(--color-brand-highlight) -80%,
    var(--color-bg)
  );
  --color-button-bg: var(--brand-gradient-button);
}
</style>
