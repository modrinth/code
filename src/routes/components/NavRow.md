<script lang="ts">
    import NavRow from "$lib/components/NavRow.svelte";
    import Example from "../_internal/components/Example.svelte"
</script>

`NavRow` works well for most horizontal navigation with less than 10 items. It can be used with paths & query params, and supports specific path level (depths).

<Example code={`
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
`}>
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
</Example>