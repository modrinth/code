<template>
  <span :class="'version-badge ' + color">
    <span class="circle" /> {{ type }}
  </span>
</template>

<script>
export default {
  name: 'Badge',
  props: {
    type: {
      type: String,
      required: true,
    },
    color: {
      type: String,
      required: true,
    },
  },
}
</script>

<style lang="scss" scoped>
.version-badge {
  display: flex;
  align-items: center;
  font-weight: bold;
  text-transform: capitalize;

  .circle {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    display: inline-block;
    margin-right: 0.25rem;
  }

  &.custom-circle {
    @media screen and (min-width: 560px) {
      .circle {
        margin-left: auto;
      }
    }
  }

  &.red {
    color: var(--color-badge-red-text);

    .circle {
      background-color: var(--color-badge-red-bg);
    }
  }

  &.green {
    color: var(--color-badge-green-text);

    .circle {
      background-color: var(--color-brand);
    }
  }

  &.yellow {
    color: var(--color-badge-yellow-text);

    .circle {
      background-color: var(--color-badge-yellow-bg);
    }
  }

  &.gray {
    color: var(--color-badge-gray-text);

    .circle {
      background-color: var(--color-badge-gray-bg);
    }
  }
}
</style>
