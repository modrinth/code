```svelte example raised
<script lang="ts">
	import { Field, Slider, TextInput } from 'omorphia'
</script>

<Field label="Favorite number" let:id>
	<Slider min="0" max="100" value="69" {id} />
</Field>
<Field label="Favorite color" helper="Pick whatever color you like the most" let:id>
	<TextInput placeholder="Enter another color..." {id} />
</Field>
```
