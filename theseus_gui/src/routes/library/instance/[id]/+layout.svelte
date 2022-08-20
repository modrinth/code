<script lang="ts">
  import VerticalNav from '$components/VerticalNav.svelte';
  import IconPackage from 'virtual:icons/lucide/package';
  import IconAdjustments from 'virtual:icons/heroicons-outline/adjustments';
  import IconFileText from 'virtual:icons/lucide/file-text';
</script>

<div class="layout-instance">
  <div class="instance-sidebar">
    <VerticalNav
      level={3}
      items={[
        {
          label: 'Mods',
          href: 'mods',
          icon: IconPackage
        },
        {
          label: 'Settings',
          href: 'settings',
          icon: IconAdjustments
        },
        {
          label: 'Logs',
          href: 'logs',
          icon: IconFileText
        }
      ]}
    />
  </div>

  <div class="layout-instance__page">
    <slot />
  </div>
</div>

<style lang="postcss">
  .layout-instance {
    display: grid;
    grid-template-columns: 224px 1fr;
    height: 100%;

    &__page {
      height: 100%;
      background-color: var(--sub-page-bg);
    }
  }

  .instance-sidebar {
    padding: 16px;
  }
</style>
