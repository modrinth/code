<script lang="ts">
  import type { SvelteComponent } from 'svelte';
  import { page } from '$app/stores';

  export let items: {
    label: string;
    /** Page href, without slash prefix */
    href: string;
    icon: SvelteComponent;
  }[];

  /** Path level in URL, zero-indexed */
  export let level = 0;

  let path: string[];
  $: path = $page.url.pathname.substring(1).split('/');
</script>

<div class="vertical-nav">
  {#each items as item (item.href)}
    <a
      class="nav-item"
      href="/{item.href}"
      class:active={path[level] === item.href}
      sveltekit:prefetch
    >
      <svelte:component this={item.icon} />
      {item.label}
    </a>
  {/each}
</div>

<style lang="postcss">
  .vertical-nav {
    display: flex;
    flex-direction: column;
    grid-gap: 0.25rem;
  }
</style>
