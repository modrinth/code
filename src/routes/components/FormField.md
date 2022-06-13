```svelte example raised
<script lang="ts">
	import { FormField, Slider, TextInput } from 'omorphia'
</script>

<FormField label="Favorite number">
	<Slider min="0" max="100" value="69" />
</FormField>
<FormField label="Favorite color">
	<TextInput placeholder="Enter another color..." />
</FormField>
```
