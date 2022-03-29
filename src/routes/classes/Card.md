### A simple example

```svelte example
<div class="card">
   <h3>Moon/Distance to Earth</h3>
   <h2>238,900 mi</h2>
   <p>
       The moon's distance from Earth affects the strength of ocean tides and the appearance of solar eclipses in our skies. The average distance between the blue planet and its only natural satellite is about 238,855 miles (384,400 kilometers), according to NASA.
   </p>
</div>
```

### A more complex example

```svelte example
<script lang="ts">
    import Button from "omorphia/components/Button.svelte";
    import IconPencil from 'virtual:icons/heroicons-outline/pencil'
    import Avatar from "omorphia/components/Avatar.svelte";
</script>

<div class="card">
   <div class="card__overlay">
       <Button color="raised"><IconPencil /> Edit</Button>
   </div>
   <div class="card__banner card__banner--short card__banner--dark" ></div>
   <Avatar size="md" floatUp/>
   <h1 class="title">Project</h1>
   <p class="summary">A project that has a description right here.</p>
</div>
```
