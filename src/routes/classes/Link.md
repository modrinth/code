### Single example

```svelte example raised
<a class="link" href="#place"> Go somewhere! </a>
```

### Group example

```svelte example raised
<script>
  import IconIssues from 'virtual:icons/heroicons-outline/exclamation'
  import IconCode from 'virtual:icons/heroicons-outline/code'
  import IconClock from 'virtual:icons/lucide/flag-triangle-right'
  import IconWiki from 'virtual:icons/heroicons-outline/book-open'
</script>

<div class="link-group">
    <a class="link" href="#issues"><IconIssues /> Issues</a>
    <a class="link" href="#source"><IconCode /> Source</a>
    <a class="link" href="#wiki"><IconWiki /> Wiki</a>
</div>
```