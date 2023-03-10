<script setup>
import { useRoute, useRouter } from 'vue-router'
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'

const route = useRoute()
const router = useRouter()
</script>

<template>
  <div class="appbar">
    <section class="navigation-controls">
      <ChevronLeftIcon @click="router.back()" />
      <ChevronRightIcon @click="router.forward()" />
      <p>{{ route.name }}</p>
    </section>
    <section class="mod-stats">
      <p>Updating 2 mods...</p>
      <p>123 mods installed</p>
    </section>
  </div>
</template>

<style lang="scss" scoped>
.appbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  height: 30px;
  border-bottom: 1px solid rgba(64, 67, 74, 0.2);
  background: var(--color-button-bg);

  .navigation-controls {
    display: inherit;
    align-items: inherit;
    justify-content: stretch;
    width: 30%;
    font-size: 1.2rem;
    margin-left: 0.5rem;

    svg {
      transition: all ease-in-out 0.1s;

      &:hover {
        filter: brightness(150%);
      }
    }

    p {
      margin-left: 0.5rem;
    }

    svg {
      margin: auto 0.2rem;
      transition: all ease-in-out 0.1s;
      cursor: pointer;

      &:hover {
        font-weight: bolder;
      }
    }
  }

  .mod-stats {
    display: inherit;
    align-items: inherit;
    justify-content: flex-end;
    width: 50%;
    font-size: 0.8rem;
    margin-right: 1rem;
  }
}
</style>
