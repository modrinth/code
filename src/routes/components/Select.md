```svelte example raised
<script lang="ts">
    import { Select } from "omorphia";
    
    let sortMethod = "downloads"
</script>

<Select
    color="raised" 
    options={[
        { value: "", label: "Relevance" },
        { value: "downloads", label: "Downloads" },
        { value: "follows", label: "Followers" },
        { value: "newest", label: "Recently created" },
        { value: "updated", label: "Recently updated" },
    ]} 
    bind:value={sortMethod}
/>
```
