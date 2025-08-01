<template>
  <div class="message-banner">
    <div class="message-banner__content" :class="cardClassByType" :aria-label="ariaLabelByType">
      <slot></slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
type MessageType = 'information' | 'warning'
const props = withDefaults(defineProps<{ messageType?: MessageType }>(), {
  messageType: 'information',
})
const cardClassByType = computed(() => `message-banner__content_${props.messageType}`)
const ariaLabelByType = computed(() => `Banner with ${props.messageType} message`)
</script>

<style lang="css" scoped>
.message-banner {
  position: relative;
  min-height: var(--font-size-2xl);

  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);
  overflow: hidden;
  outline: 2px solid transparent;
  outline-offset: -2px;

  margin-bottom: var(--spacing-card-md);

  box-shadow: var(--shadow-card);

  line-height: 1.5;
  min-height: 0;
}

:slotted(a) {
  /* Uses active color to increase contrast */
  color: var(--color-link-active);
  text-decoration: underline;
}

.message-banner__content {
  padding: var(--spacing-card-md) var(--spacing-card-lg);
}

.message-banner__content_warning {
  border-left: 0.5rem solid var(--color-warning-banner-side);
  background-color: var(--color-warning-banner-bg);
  color: var(--color-warning-banner-text);
}

.message-banner__content_information {
  border-left: 0.5rem solid var(--color-info-banner-side);
  background-color: var(--color-info-banner-bg);
  color: var(--color-info-banner-text);
}
</style>
