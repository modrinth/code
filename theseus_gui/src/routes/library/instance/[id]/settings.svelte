<script context="module" lang="ts">

</script>

<script lang="ts">
    import GlobalSettings from "$components/GlobalSettings.svelte"
    import TitledSection from "$components/TitledSection.svelte"

    export let overrideGlobalSettings = false
</script>

<div class="section">
    <TitledSection title="Override global settings" toggleable=true>
        <GlobalSettings />
    </TitledSection>
</div>

<style lang="postcss">
</style>
