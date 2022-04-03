<script lang="ts">
    export let title: string;
</script>

<div class="card-row">
    <div class="card-row__title">{title}</div>
    <div class="card-row__items">
        <slot />
    </div>
</div>


<style lang="postcss">
    .card-row {
        display: flex;
        flex-direction: column;
        padding: 1rem 0;
        grid-gap: 1rem;

        &__title {
            padding: 0 1rem;
            display: flex;
            grid-gap: 1rem;
            align-items: center;

            &::after {
                flex: 1 1;
                content: " ";
                background-color: hsla(0,0%,100%,0.2);
                height: 0.2rem;
                border-radius: var(--rounded-max);
            }
        }

        &__items {
            display: flex;
            grid-gap: 1rem;
            align-items: flex-start;
            overflow-x: auto;
            padding: 0 1rem;

            /* Hide scrollbar */
            -ms-overflow-style: none;
            scrollbar-width: none;
            &::-webkit-scrollbar {
                display: none;
            }
        }

        &:nth-of-type(even) {
            background-color: hsla(0,0%,0%,0.2);
        }
    }
</style>