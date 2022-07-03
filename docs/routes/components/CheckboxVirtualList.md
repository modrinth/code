```svelte example raised
<script lang="ts">
	import { CheckboxVirtualList } from 'omorphia'
	import IconStar from 'virtual:icons/heroicons-outline/star'

	let options = Array(100)
		.fill({})
		.map((option, index) => ({
			label: 'Star-' + index,
			icon: IconStar,
			value: index,
		}))

	let selected = [22, 24]
</script>

<CheckboxVirtualList bind:value={selected} {options} />

Selected: {selected}
```
