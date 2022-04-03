<script lang="ts">
    import { Svrollbar } from 'svrollbar'

    let viewport: Element
    let contents: Element
</script>

<div class="page">
    <div bind:this={viewport} class="viewport">
        <div bind:this={contents} class="contents">
            <slot />
        </div>
    </div>
    <Svrollbar {viewport} {contents} />
</div>

<style lang="postcss">
    .page {
        position: relative;
        width: 100%;
        overflow: hidden;

        --svrollbar-track-width: 20px;
        --svrollbar-track-opacity: 0;

        --svrollbar-thumb-width: 8px;
        --svrollbar-thumb-background: hsla(216,5%,60%);
        --svrollbar-thumb-opacity: 0.9;
    }

    .viewport {
        position: relative;
        width: 100%;
        height: calc(100vh - 2.5rem);
        overflow-y: scroll;
        overflow-x: hidden;

        /* hide scrollbar */
        -ms-overflow-style: none;
        scrollbar-width: none;

        &::-webkit-scrollbar {
            /* hide scrollbar */
            display: none;
        }

        .contents {
            height: 100%;
        }
    }

    :global(.v-thumb) {
        margin: 4px auto 4px auto !important;
    }
</style>