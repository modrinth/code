```svelte example raised
<script lang="ts">
	import { CheckboxVirtualList } from 'omorphia'
	import IconStar from 'virtual:icons/heroicons-outline/star'
	import { uniqueId } from 'omorphia/utils/uniqueId'

	let options = Array(100)
		.fill({})
		.map((option) => ({
			label: 'Star-' + uniqueId(),
			icon: IconStar,
			value: uniqueId(),
		}))

	let selected = ['2', '6']
</script>

<CheckboxVirtualList bind:value={selected} {options} />

Selected: {selected}
```
