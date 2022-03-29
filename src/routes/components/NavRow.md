`NavRow` works well for most horizontal navigation with less than 10 items. It can be used with paths & query params, and supports specific path level (depths).

```svelte example
<script lang="ts">
    import NavRow from "omorphia/components/NavRow.svelte";
</script>

<NavRow
    level={1}
    links={[
        {
            href: '/Button',
            label: 'Button'
        },
        {
            href: '/Link',
            label: 'Link'
        },
        {
            href: '/NavRow',
            label: 'NavRow'
        }
    ]}>
    Click for fun
</NavRow>
```