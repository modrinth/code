### Single Example

```svelte example raised
<script>
    import IconStar from 'virtual:icons/heroicons-outline/star'
</script>

<div class="stat">
  <IconStar/>
  123K stars
</div>
```

### Group Example

```svelte example raised
<script>
    import IconDownload from 'virtual:icons/heroicons-outline/download'
    import IconHeart from 'virtual:icons/heroicons-outline/heart'
</script>

<div class="stat-group">
    <div class="stat">
      <IconDownload/>
      4.1B downloads
    </div>
    <div class="stat stat--light">
      <IconHeart/>
      3 followers
    </div>
</div>
```
