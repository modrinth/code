<template>
  <div ref="button" class="button-base avatar-button" :class="{ highlighted: showDemo }">
    <Avatar src="https://launcher-files.modrinth.com/assets/steve_head.png" />
  </div>
  <transition name="fade">
    <div v-if="showDemo" class="card-section">
      <Card ref="card" class="fake-account-card expanded highlighted">
        <div class="selected account">
          <Avatar size="xs" src="https://launcher-files.modrinth.com/assets/steve_head.png" />
          <div>
            <h4>Modrinth</h4>
            <p>Selected</p>
          </div>
          <Button v-tooltip="'Log out'" icon-only color="raised">
            <TrashIcon />
          </Button>
        </div>
        <Button>
          <PlusIcon />
          Add account
        </Button>
      </Card>
      <slot />
    </div>
  </transition>
</template>

<script setup>
import { Avatar, Button, Card, PlusIcon, TrashIcon } from 'omorphia'

defineProps({
  showDemo: {
    type: Boolean,
    default: false,
  },
})
</script>

<style scoped lang="scss">
.selected {
  background: var(--color-brand-highlight);
  border-radius: var(--radius-lg);
  color: var(--color-contrast);
  gap: 1rem;
}

.logged-out {
  background: var(--color-bg);
  border-radius: var(--radius-lg);
  gap: 1rem;
}

.account {
  width: max-content;
  display: flex;
  align-items: center;
  text-align: left;
  padding: 0.5rem 1rem;

  h4,
  p {
    margin: 0;
  }
}

.card-section {
  position: absolute;
  top: 0.5rem;
  left: 5.5rem;
  z-index: 9;
  display: flex;
  flex-direction: column;
}

.fake-account-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 1rem;
  border: 1px solid var(--color-button-bg);
  width: max-content;
  user-select: none;
  -ms-user-select: none;
  -webkit-user-select: none;

  &.hidden {
    display: none;
  }

  &.isolated {
    position: relative;
    left: 0;
    top: 0;
  }
}

.accounts-title {
  font-size: 1.2rem;
  font-weight: bolder;
}

.account-group {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.option {
  width: calc(100% - 2.25rem);
  background: var(--color-raised-bg);
  color: var(--color-base);
  box-shadow: none;

  img {
    margin-right: 0.5rem;
  }
}

.icon {
  --size: 1.5rem !important;
}

.account-row {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  vertical-align: center;
  justify-content: space-between;
  padding-right: 1rem;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.avatar-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-base);
  background-color: var(--color-raised-bg);
  border-radius: var(--radius-md);
  width: 100%;
  text-align: left;

  &.expanded {
    border: 1px solid var(--color-button-bg);
    padding: 1rem;
  }
}

.avatar-text {
  margin: auto 0 auto 0.25rem;
  display: flex;
  flex-direction: column;
}

.text {
  width: 6rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.accounts-text {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin: 0;
}
</style>
