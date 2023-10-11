<script setup>
const options = [
  {
    'id': 'like',
    'color': 'primary',
    'action': () => {}
  },
  {
    'id': 'report',
    'action': () => {}
  },
  {
    'id': 'delete',
    'color': 'danger',
    'action': () => {}
  }
]
</script>

# Overflow Menu
<DemoContainer>
  <OverflowMenu :options="options" class="btn">
    More options...
    <template #like>
      <HeartIcon /> Like
    </template>
    <template #report>
      <ReportIcon /> Report
    </template>
    <template #delete>
      <TrashIcon /> Delete
    </template>
  </OverflowMenu>
</DemoContainer>

```vue
<OverflowMenu :options="options" class="btn">
  More options...
  <template #like>
    <HeartIcon /> Like
  </template>
  <template #report>
    <ReportIcon /> Report
  </template>
  <template #delete>
    <TrashIcon /> Delete
  </template>
</OverflowMenu>
```
