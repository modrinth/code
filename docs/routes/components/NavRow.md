`NavRow` works well for most horizontal navigation with less than 10 items. It can be used with paths & query params, and supports specific path level (depths).

### Query example

```svelte example
<script lang="ts">
	import { NavRow } from 'omorphia'
</script>

<div class="card card--strip card--pad-x">
	<NavRow
		level={1}
		query={'tab'}
		links={[
			{
				href: '',
				label: 'All',
			},
			{
				href: 'mods',
				label: 'Mods',
			},
			{
				href: 'modpacks',
				label: 'Modpacks',
			},
		]} />
</div>
```

### Route example

```svelte example
<script lang="ts">
	import { NavRow } from 'omorphia'
</script>

<div class="card card--strip card--pad-x">
	<NavRow
		level={1}
		links={[
			{
				href: '/Button',
				label: 'Button',
			},
			{
				href: '/Chips',
				label: 'Chips',
			},
			{
				href: '/NavRow',
				label: 'NavRow',
			},
		]} />
</div>
```
