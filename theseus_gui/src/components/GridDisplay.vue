<script setup>
import Instance from '@/components/ui/Instance.vue'
import { ref } from 'vue'

const props = defineProps({
  instances: {
    type: Array,
    default() {
      return []
    },
  },
  news: {
    type: Array,
    default() {
      return []
    },
  },
  label: {
    type: String,
    default: '',
  },
  canPaginate: Boolean,
})
const modsRow = ref(null)
</script>
<template>
  <div class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <hr />
    </div>
    <section ref="modsRow" class="instances">
      <Instance
        v-for="instance in props.instances"
        :key="instance.id"
        display="card"
        :instance="instance"
      />
    </section>
  </div>
</template>
<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 1rem;

  .header {
    display: flex;
    justify-content: space-between;
    align-items: inherit;
    width: 100%;
    margin-bottom: 1rem;
    gap: 1rem;

    p {
      font-size: 1rem;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    hr {
      background-color: var(--color-gray);
      height: 1px;
      width: 100%;
      border: none;
    }

    .pagination {
      display: inherit;
      align-items: inherit;

      svg {
        background: var(--color-raised-bg);
        border-radius: var(--radius-lg);
        width: 1.3rem;
        height: 1.2rem;
        cursor: pointer;
        margin-right: 0.5rem;
        transition: all ease-in-out 0.1s;

        &:hover {
          filter: brightness(150%);
        }
      }
    }
  }

  section {
    display: flex;
    align-items: inherit;
    transition: all ease-in-out 0.4s;
    gap: 1rem;
  }

  .instances {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    width: 100%;
    gap: 1rem;
    margin-right: auto;
    margin-top: 0.8rem;
    scroll-behavior: smooth;
    overflow-y: auto;
  }
}

.dark-mode {
  .row {
    &:nth-child(odd) {
      background-color: rgb(30, 31, 34);
    }
  }
}
</style>
