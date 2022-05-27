<script lang="ts">
    // TODO: Add fade out styling on top and bottom

    import Checkbox from './Checkbox.svelte';
    import VirtualList from 'svelte-tiny-virtual-list';
    import type { Option } from './types';

    /** Height in pixels of list */
    export let height = 200;

    export let value = [];
    export let options: Option[] = [];

    const handleChange = (e, key) => {
        if (e.target.checked) {
            if (!value) value = [];
            value = [key, ...value];
        } else {
            value = value.filter((it) => key !== it);
        }
    };
</script>

<VirtualList width="100%" {height} itemCount={options.length} itemSize={26}>
    <div
        slot="item"
        let:index
        let:style
        {style}
        style:padding-bottom={options.length - 1 === index ? '2.5rem' : ''}
    >
        {@const option = options[index]}
        <Checkbox
            checked={value && value.includes(option.value)}
            on:change={(e) => handleChange(e, option.value)}
        >
            {#if option.icon && typeof option.icon === 'string'}
                {@html option.icon}
            {:else if option.icon}
                <svelte:component this={option.icon} />
            {/if}
            {option.label}
        </Checkbox>
    </div>
</VirtualList>
