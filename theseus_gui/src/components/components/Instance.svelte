<script lang="ts">
    import IconPlayFilled from "virtual:icons/carbon/play-filled-alt"

    export let title: string;
    export let id: string;
    export let version: string;
    export let modpack = false;
    export let image: string;
</script>

<a class="instance" href="/library/instance/{id}"
   style:background-image="linear-gradient(5deg, hsla(0,0%,0%,0.8) 0%, hsla(0,0%,0%,0) 100%), url('{image}')">
    <div class="instance__version">{version}</div>
    <div class="instance__title">{title}</div>
    <button class="play-button">
       <IconPlayFilled />
    </button>
</a>

<style lang="postcss">
    .instance {
        --size: 8rem;
        min-width: var(--size);
        min-height: var(--size);
        border-radius: var(--rounded);
        overflow: hidden;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        padding: 0.5rem;
        box-shadow: var(--shadow-raised) var(--shadow-inset);
        position: relative;

        background-size: cover;

        &__title {
            text-overflow: ellipsis;
            overflow: hidden;
            white-space: nowrap;
        }

        &__version {
            color: var(--color-text);
            font-size: var(--font-size-sm);
        }

        &:hover .play-button {
            visibility: visible;
            opacity: 1;
            transform: none;
        }

        .play-button {
            position: absolute;
            bottom: 0.5rem;
            right: 0.5rem;
            background-color: var(--color-brand);
            width: 2.5rem;
            aspect-ratio: 1 / 1;
            border-radius: var(--rounded-max);
            display: flex;
            align-items: center;
            justify-content: center;
            visibility: hidden;
            opacity: 0;
            transition-property: opacity, visibility, transform;
            transition-duration: 0.2s;
            transition-timing-function: ease-in-out;
            transform: translateY(1rem);

            :global(svg) {
                width: 1.25rem;
                height: auto;
                color: black;
            }
        }
    }
</style>
