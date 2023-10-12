# Overflow Menu
<DemoContainer>
  <OverflowMenu :options="[
  {
    'id': 'play',
    'color': 'primary',
    'action': () => {},
    'hoverFilledOnly': true
  },
  { divider: true },
  {
    'id': 'duplicate',
    'action': () => {}
  },
  {
    'id': 'report',
    'action': () => {}
  },
  {
    'id': 'remain',
    'action': () => {},
    'remainOnClick': true,
  },
  { divider: true },
  {
    'id': 'delete',
    'color': 'danger',
    'action': () => {},
    'hoverFilled': true,
  }
]" class="btn">
    More options...
    <template #play>
      <PlayIcon /> Play
    </template>
    <template #duplicate>
      <CopyIcon /> Duplicate
    </template>
    <template #report>
      <ReportIcon /> Report
    </template>
    <template #remain>
      <ClearIcon /> I shall remain
    </template>
    <template #delete>
      <TrashIcon /> Delete
    </template>
  </OverflowMenu>
</DemoContainer>

```vue
<OverflowMenu
  class="btn"
  :options="[
    {
      'id': 'play',
      'color': 'primary',
      'action': () => {},
      'hoverFilledOnly': true
    },
    { divider: true },
    {
      'id': 'duplicate',
      'action': () => {}
    },
    {
      'id': 'report',
      'action': () => {}
    },
    {
      'id': 'remain',
      'action': () => {},
      'remainOnClick': true,
    },
    { divider: true },
    {
      'id': 'delete',
      'color': 'danger',
      'action': () => {},
      'hoverFilled': true,
    }
]">
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
