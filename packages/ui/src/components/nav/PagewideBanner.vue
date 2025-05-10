<template>
  <div
    :class="['banner-grid relative border-b-2 border-solid border-0', containerClasses[variant]]"
  >
    <div
      :class="[
        'grid-area-[title] flex items-center gap-2 font-bold text-[var(--font-size-md)]',
        iconClasses[variant],
      ]"
    >
      <IssuesIcon
        v-if="variant === 'warning' || variant === 'error'"
        aria-hidden="true"
        class="w-6 h-6 flex-shrink-0"
      />
      <InfoIcon v-if="variant === 'info'" aria-hidden="true" class="w-6 h-6 flex-shrink-0" />
      <slot name="title" />
    </div>

    <div class="grid-area-[description] flex flex-col gap-[var(--gap-md)]">
      <slot name="description" />
    </div>

    <div v-if="$slots.actions" class="grid-area-[actions]">
      <slot name="actions" />
    </div>

    <div v-if="$slots.actions_right" class="grid-area-[actions_right]">
      <slot name="actions_right" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { InfoIcon, IssuesIcon } from '@modrinth/assets'

defineProps<{
  variant: 'error' | 'warning' | 'info'
}>()

const containerClasses = {
  error: 'bg-banners-error-bg text-banners-error-text border-banners-error-border',
  warning: 'bg-banners-warning-bg text-banners-warning-text border-banners-warning-border',
  info: 'bg-banners-info-bg text-banners-info-text border-banners-info-border',
}

const iconClasses = {
  error: 'text-brand-red',
  warning: 'text-brand-orange',
  info: 'text-brand-blue',
}
</script>

<style scoped>
.banner-grid {
  display: grid;
  gap: 0.5rem;
  grid-template-areas:
    'title         actions_right'
    'description   actions_right'
    'actions       actions_right';
  padding-block: var(--gap-xl);
  padding-inline: max(calc((100% - 80rem) / 2 + var(--gap-md)), var(--gap-xl));
}

.grid-area-\[title\] {
  grid-area: title;
}
.grid-area-\[description\] {
  grid-area: description;
}
.grid-area-\[actions\] {
  grid-area: actions;
}
.grid-area-\[actions_right\] {
  grid-area: actions_right;
}

.banner-grid a {
  @apply underline text-current;
}
</style>
