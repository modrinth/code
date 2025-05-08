<template>
  <div
    :class="[
      `banner-grid relative before:content-[''] before:absolute before:inset-0 before:z-[1] [&>*]:z-[6] border-solid border-0`,
      containerClasses[variant]
    ]"
  >
    <div
      :class="[
        'grid-area-[title] flex items-center gap-2 font-bold text-[var(--font-size-md)]',
        iconClasses[variant]
      ]"
    >
      <IssuesIcon
        v-if="variant === 'warning' || variant === 'error'"
        aria-hidden="true"
        class="w-6 h-6 flex-shrink-0"
        :class="iconClasses[variant]"
      />
      <InfoIcon
        v-if="variant === 'info'"
        aria-hidden="true"
        class="w-6 h-6 flex-shrink-0"
        :class="iconClasses[variant]"
      />
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

const props = defineProps<{
  variant: 'error' | 'warning' | 'info'
}>()

const containerClasses = {
  error:   'bg-brand-red-50 border-b-2 border-b-brand-red before:bg-brand-red-900',
  warning: 'bg-brand-orange-50 border-b-2 border-b-brand-orange before:bg-brand-orange-900',
  info:    'bg-brand-blue-50 border-b-2 border-b-brand-blue before:bg-brand-blue-900',
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

.grid-area-\[title\] { grid-area: title; }
.grid-area-\[description\] { grid-area: description; }
.grid-area-\[actions\] { grid-area: actions; }
.grid-area-\[actions_right\] { grid-area: actions_right; }

.banner-grid a {
  @apply underline text-current;
}
</style>
