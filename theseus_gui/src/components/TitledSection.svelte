<script lang="ts">
  import { Checkbox } from 'omorphia';

  export let title: string;
  export let toggleable: boolean = false;

  export let enabled: boolean = false;
</script>

<div class="section">
  <div class="section__title">
    {#if toggleable}<Checkbox bind:checked={enabled}>{title}</Checkbox>
    {:else}{title}
    {/if}
  </div>
  <div class="section__items">
    {#if !toggleable || enabled}<slot />{/if}
  </div>
</div>

<style lang="postcss">
  .section {
    display: flex;
    flex-direction: column;
    padding: 1rem;
    grid-gap: 1rem;

    &__title {
      display: flex;
      grid-gap: 1rem;
      align-items: center;

      &::after {
        flex: 1 1;
        content: ' ';
        background-color: hsla(0, 0%, 100%, 0.2);
        height: 0.2rem;
        border-radius: var(--rounded-max);
      }
    }

    &__items {
      display: flex;
      flex-direction: column;
      grid-gap: 1rem;
      padding: 0 8px;
    }
  }
</style>
