<script lang="ts">
    import "$lib/styles/variables.postcss";
    import "$lib/styles/themes/light.postcss";
</script>

<main>
    <slot></slot>
</main>

<style lang="postcss">
    main {
      font-size: var(--body-font-size);
      font-family: var(--body-font);
    }
</style>