# Categories

<DemoContainer>
 <Categories
    :categories="[{
        name: 'magic',
        icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><path d=\'M15 4V2\'></path><path d=\'M15 16v-2\'></path><path d=\'M8 9h2\'></path><path d=\'M20 9h2\'></path><path d=\'M17.8 11.8 19 13\'></path><path d=\'M15 9h0\'></path><path d=\'M17.8 6.2 19 5\'></path><path d=\'m3 21 9-9\'></path><path d=\'M12.2 6.2 11 5\'></path></svg>',
      },
      {
        icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><rect x=\'2\' y=\'7\' width=\'20\' height=\'14\' rx=\'2\' ry=\'2\'/><path d=\'M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16\'/></svg>',
        name: 'utility',
      },
      {
        icon: '<svg xmlns=\'http://www.w3.org/2000/svg\' xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'/>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'23\' d=\'m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8\' transform=\'matrix(.08671 0 0 .0867 -49.8 -56)\'/>\n</svg>',
        name: 'fabric',
      },
      {
        icon: '<svg xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-miterlimit=\'1.5\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'></path>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' d=\'M2 7.5h8v-2h12v2s-7 3.4-7 6 3.1 3.1 3.1 3.1l.9 3.9H5l1-4.1s3.8.1 4-2.9c.2-2.7-6.5-.7-8-6Z\'></path>\n</svg>',
        name: 'forge'
      }]"
    type="mod"
    class="tags"
  />
</DemoContainer>

```vue
<Categories
  :categories="[{
      name: 'magic',
      icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><path d=\'M15 4V2\'></path><path d=\'M15 16v-2\'></path><path d=\'M8 9h2\'></path><path d=\'M20 9h2\'></path><path d=\'M17.8 11.8 19 13\'></path><path d=\'M15 9h0\'></path><path d=\'M17.8 6.2 19 5\'></path><path d=\'m3 21 9-9\'></path><path d=\'M12.2 6.2 11 5\'></path></svg>',
    },
    {
      icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><rect x=\'2\' y=\'7\' width=\'20\' height=\'14\' rx=\'2\' ry=\'2\'/><path d=\'M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16\'/></svg>',
      name: 'utility',
    },
    {
      icon: '<svg xmlns=\'http://www.w3.org/2000/svg\' xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'/>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'23\' d=\'m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8\' transform=\'matrix(.08671 0 0 .0867 -49.8 -56)\'/>\n</svg>',
      name: 'fabric',
    },
    {
      icon: '<svg xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-miterlimit=\'1.5\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'></path>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' d=\'M2 7.5h8v-2h12v2s-7 3.4-7 6 3.1 3.1 3.1 3.1l.9 3.9H5l1-4.1s3.8.1 4-2.9c.2-2.7-6.5-.7-8-6Z\'></path>\n</svg>',
      name: 'forge'
    }]"
  type="mod"
  class="tags"
/>
```
