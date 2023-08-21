# Analytics

<DemoContainer>
    <LineChart 
        :data="{
          labels: [
            '2021-01-01', '2021-01-02', '2021-01-03', '2021-01-04', '2021-01-05',
            '2021-01-06', '2021-01-07', '2021-01-08', '2021-01-09', '2021-01-10',
            '2021-01-11', '2021-01-12', '2021-01-13', '2021-01-14', '2021-01-15',
            '2021-01-16', '2021-01-17'
          ],
          data: [
            {
              title: 'Spirit',
              color: 16711680,
              data: [120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240, 250, 260, 270, 280],
            },
            {
              title: 'Ad Astra',
              color: 65280,
              data: [150, 155, 160, 165, 170, 175, 180, 185, 190, 195, 200, 205, 210, 215, 220, 225, 230],
            },
            {
              title: 'Tempad',
              color: 255,
              data: [180, 182, 184, 186, 188, 190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212],
            },
          ]
        }"
    />
    <PieChart
        :data="{
            title: 'Downloads',
            data: [
                {
                    title: 'Spirit',
                    color: 16711680, // Red in decimal (equivalent to #FF0000 in hexadecimal)
                    data: 120, // Example download numbers for the three dates
                },
                {
                    title: 'Ad Astra',
                    color: 65280, // Green in decimal (equivalent to #00FF00 in hexadecimal)
                    data: 150, // Example download numbers for the three dates
                },
                {
                    title: 'Tempad',
                    color: 255, // Blue in decimal (equivalent to #0000FF in hexadecimal)
                    data: 180, // Example download numbers for the three dates
                },
            ],    
        }"
    />
</DemoContainer>

```vue
<LineChart 
  :formatLabel="(label) => doFormattingThings(label)" 
  :data="{
          labels: [
            '2021-01-01', '2021-01-02', '2021-01-03', '2021-01-04', '2021-01-05',
            '2021-01-06', '2021-01-07', '2021-01-08', '2021-01-09', '2021-01-10',
            '2021-01-11', '2021-01-12', '2021-01-13', '2021-01-14', '2021-01-15',
            '2021-01-16', '2021-01-17'
          ],
          data: [
            {
              title: 'Spirit',
              color: 16711680,
              data: [120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240, 250, 260, 270, 280],
            },
            {
              title: 'Ad Astra',
              color: 65280,
              data: [150, 155, 160, 165, 170, 175, 180, 185, 190, 195, 200, 205, 210, 215, 220, 225, 230],
            },
            {
              title: 'Tempad',
              color: 255,
              data: [180, 182, 184, 186, 188, 190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212],
            },
          ]
        }"
/>
```
```vue
<PieChart
  :data="{
            title: 'Downloads',
            data: [
                {
                    title: 'Spirit',
                    color: 16711680, // Red in decimal (equivalent to #FF0000 in hexadecimal)
                    data: 120, // Example download numbers for the three dates
                },
                {
                    title: 'Ad Astra',
                    color: 65280, // Green in decimal (equivalent to #00FF00 in hexadecimal)
                    data: 150, // Example download numbers for the three dates
                },
                {
                    title: 'Tempad',
                    color: 255, // Blue in decimal (equivalent to #0000FF in hexadecimal)
                    data: 180, // Example download numbers for the three dates
                },
            ],    
        }"
/>
```
