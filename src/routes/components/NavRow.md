`NavRow` works well for most horizontal navigation with less than 10 items. It can be used with paths & query params, and supports specific path level (depths).

```svelte example raised
<script lang="ts">
    import { NavRow } from "omorphia";
</script>

<NavRow
    level={1}
    links={[
        {
            href: '/Button',
            label: 'Button'
        },
        {
            href: '/Chips',
            label: 'Chips'
        },
        {
            href: '/NavRow',
            label: 'NavRow'
        }
    ]}>
    Click for fun
</NavRow>
```