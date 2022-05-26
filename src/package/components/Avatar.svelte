<script lang="ts">
    // TODO: Make square icon `md` more rounded

    import { onMount } from 'svelte';
    import { classCombine } from '../utils/classCombine';

    /** Optional, as a default icon will be substituted if no image was specified */
    export let src: string | undefined;
    export let size: 'xs' | 'sm' | 'md' | 'lg';
    export let circle = false;
    export let floatUp = false;

    let className: string;
    $: className = classCombine([
        'avatar',
        circle && 'avatar--circle',
        `avatar--size-${size}`,
        floatUp && 'avatar--float-up',
    ]);

    let img;

    onMount(() => {
        if (img && img.naturalWidth) {
            const isPixelated = () => {
                if (img.naturalWidth < 96 && img.naturalWidth > 0) {
                    img.style = 'image-rendering: pixelated;';
                }
            };

            if (img.naturalWidth) {
                isPixelated();
            } else {
                img.onload = isPixelated;
            }
        }
    });
</script>

{#if src}
    <img {src} bind:this={img} class={className} alt="" />
{:else}
    <svg
        class={className}
        xml:space="preserve"
        fill-rule="evenodd"
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-miterlimit="1.5"
        clip-rule="evenodd"
        viewBox="0 0 104 104"
    >
        <path fill="none" d="M0 0h103.4v103.4H0z" />
        <path
            fill="none"
            stroke="#9a9a9a"
            stroke-width="5"
            d="M51.7 92.5V51.7L16.4 31.3l35.3 20.4L87 31.3 51.7 11 16.4 31.3v40.8l35.3 20.4L87 72V31.3L51.7 11"
        />
    </svg>
{/if}

<style lang="postcss">
    .avatar {
        border-radius: var(--rounded);
        box-shadow: var(--shadow-inset-lg), var(--shadow-raised-lg);
        height: var(--size);
        width: var(--size);
        background-color: var(--color-button-bg);

        &--size {
            &-xs {
                --size: 2.25rem;
                box-shadow: var(--shadow-inset), var(--shadow-raised);
            }

            &-sm {
                --size: 3rem;
                box-shadow: var(--shadow-inset), var(--shadow-raised);
            }

            &-md {
                --size: 6rem;
                border-radius: var(--rounded-lg);
            }

            &-lg {
                --size: 9rem;
                border-radius: var(--rounded-lg);
            }
        }

        &--float-up {
            margin-top: calc(var(--size) * (-2 / 3));
            z-index: 1;
        }

        &--circle {
            border-radius: 50%;
        }
    }
</style>
