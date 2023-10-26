# Analytics

<DemoContainer>
<client-only>
  <Chart 
    name="Chart"
    type="bar"
    :stacked="true"
    :labels="[
      '2021-01-01', '2021-01-02', '2021-01-03', '2021-01-04', '2021-01-05',
      '2021-01-06', '2021-01-07', '2021-01-08', '2021-01-09', '2021-01-10',
      '2021-01-11', '2021-01-12', '2021-01-13', '2021-01-14', '2021-01-15',
      '2021-01-16', '2021-01-17'
    ]"
    :data="[
      {
        name: 'Spirit',
        data: [120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240, 250, 260, 270, 280],
      },
      {
        name: 'Ad Astra',
        data: [150, 155, 160, 165, 170, 175, 180, 185, 190, 195, 200, 205, 210, 215, 220, 225, 230],
      },
      {
        name: 'Tempad',
        data: [180, 182, 184, 186, 188, 190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212],
      },
    ]"  
    :colors="['#FF0000', '#00FF00', '#0000FF']"
    suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
  />
</client-only>
</DemoContainer>
<DemoContainer>
<client-only>
  <Chart 
    name="Chart"
    type="line"
    :labels="[
      '2021-01-01', '2021-01-02', '2021-01-03', '2021-01-04', '2021-01-05',
      '2021-01-06', '2021-01-07', '2021-01-08', '2021-01-09', '2021-01-10',
      '2021-01-11', '2021-01-12', '2021-01-13', '2021-01-14', '2021-01-15',
      '2021-01-16', '2021-01-17'
    ]"
    :data="[
      {
        name: 'Spirit',
        data: [120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240, 250, 260, 270, 1280],
      },
      {
        name: 'Ad Astra',
        data: [150, 155, 160, 165, 170, 175, 180, 185, 190, 195, 200, 205, 210, 215, 220, 225, 1230],
      },
      {
        name: 'Tempad',
        data: [180, 182, 184, 186, 188, 190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212],
      },
    ]"  
    :colors="['#FF0000', '#00FF00', '#0000FF']"
    suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
  />
</client-only>
</DemoContainer>
<DemoContainer>
<client-only>
  <Chart 
    name="Chart"
    :labels="[
      '2021-01-01', '2021-01-02', '2021-01-03', '2021-01-04', '2021-01-05',
      '2021-01-06', '2021-01-07', '2021-01-08', '2021-01-09', '2021-01-10',
      '2021-01-11', '2021-01-12', '2021-01-13', '2021-01-14', '2021-01-15',
      '2021-01-16', '2021-01-17'
    ]"
    :data="[
      {
        name: 'Downloads',
        data: [120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240, 250, 260, 270, 280],
      },
      {
        name: 'Revenue',
        data: [150, 155, 160, 165, 170, 175, 180, 185, 190, 195, 200, 205, 210, 215, 220, 225, 230],
      },
      {
        name: 'Page views',
        data: [180, 182, 184, 186, 188, 190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212],
      },
    ]"  
    hide-total
    suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
  >
  Slot for title stuff
  <Chips :items="['option 1', 'option 3']" />
  <template #toolbar>
    <Button>
      <PlusIcon />
      Slot for toolbar stuff
    </Button>
  </template>
</Chart>
</client-only>
</DemoContainer>
<DemoContainer>
<client-only>
<div style="display: grid; grid-template-columns: 1fr 1fr; column-gap: var(--gap-md);">
  <CompactChart 
    v-for="i in 4"
    title="Downloads"
    value="10,230"
    :labels="[
      '2021-01-01', '2021-01-02', '2021-01-03', '2021-01-04', '2021-01-05',
      '2021-01-06', '2021-01-07', '2021-01-08', '2021-01-09', '2021-01-10',
      '2021-01-11', '2021-01-12', '2021-01-13', '2021-01-14', '2021-01-15',
      '2021-01-16', '2021-01-17'
    ]"
    :data="[
      {
        name: 'Downloads',
        data: [240, 180, 210, 160, 250, 130, 220, 270, 120, 260, 200, 230, 140, 280, 190, 150, 170],
      }
    ]"
    suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
  />
</div>
</client-only>
</DemoContainer>

```vue
<Chart
  name="Chart name"
  :labels="['array', 'of', 'labels', 'for', 'x-axis', 'typically', 'dates']"
  :data="[
      {
        name: 'Spirit',
        data: ['array', 'of', 'data', 'equal', 'length', 'to', 'x-axis'],
      },
      ...
    ]"
  :colors="['array', 'of', 'colors', 'for', 'each', 'series/dataset']"
  prefix="string or svg icon to append to each data point"
  suffix="string or svg icon to append to each data point"
  type="bar|line"
  :stacked="true|false (default: false) (determines whether or not values overlap/sidebyside instead of stacked)"
  :hideTotal="true|false (default: false) (hide total value in tooltip)"
  :hideToolbar="true|false (default: false) (hide toolbar)"
  :hideLegend="true|false (default: false) (hide legend)"
>
  ... slot for title stuff
  <template #toolbar>
    ... slot for toolbar stuff
  </template>
</Chart>
```

```vue
<CompactChart
  title="Chart title"
  value="Chart value"
  :labels="['array', 'of', 'labels', 'for', 'x-axis', 'typically', 'dates']"
  :data="[
      {
        name: 'Spirit',
        data: ['array', 'of', 'data', 'equal', 'length', 'to', 'x-axis'],
      },
      ...
  ]"
  prefix="string or svg icon to append to each data point"
  suffix="string or svg icon to append to each data point"
/>
```
