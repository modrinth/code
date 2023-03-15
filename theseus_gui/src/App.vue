<script setup>
import { watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import { useTheming } from '@/store/state'
import Navigation from '@/components/ui/Navigation.vue'
import { toggleTheme } from '@/helpers/theme'

const route = useRoute()
const router = useRouter()

const theme = useTheming()

toggleTheme(theme.darkTheme)

watch(theme, (newState) => {
  toggleTheme(newState.darkTheme)
})
</script>

<template>
  <div class="container">
    <Navigation class="navigation" />
    <div class="view">
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
      <RouterView />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.container {
  min-width: 100%;
  min-height: 100vh;
  overflow-x: hidden;

  .navigation {
    position: fixed;
  }

  .view {
    height: 100%;
    margin-left: 210px;

    .appbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      width: 100%;
      height: 30px;
      border-bottom: 1px solid rgba(64, 67, 74, 0.2);
      background: var(--color-button-bg);
      padding: 1.2rem;

      .navigation-controls {
        display: inherit;
        align-items: inherit;
        justify-content: stretch;
        width: 30%;
        font-size: 0.9rem;

        svg {
          width: 18px;
          transition: all ease-in-out 0.1s;

          &:hover {
            filter: brightness(150%);
          }
        }

        p {
          margin-left: 0.3rem;
        }

        svg {
          margin: auto 0.1rem;
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

        p:nth-child(1) {
          margin-right: 0.55rem;
        }
      }
    }
  }
}
</style>
