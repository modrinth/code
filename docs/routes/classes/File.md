```svelte example raised
<script lang="ts">
	import IconFile from 'virtual:icons/lucide/file'
	import { Button } from 'omorphia'
</script>

<div class="file file--primary">
	<div class="file__tab">
		<IconFile />
		<div class="file__tab__name"><b>cool-mod.jar</b></div>
		<Button raised>Download</Button>
	</div>
</div>
```
