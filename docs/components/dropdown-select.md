# Dropdown

<DemoContainer>
  <DropdownSelect
    id="report-type"
    v-model="reportType"
    :options="['Daily', 'Weekly', 'Monthly']"
    defaultValue="Choose Frequency"
  />
</DemoContainer>

```vue
<DropdownSelect
  id="report-type"
  v-model="reportType"
  :options="['Daily', 'Weekly', 'Monthly']"
  defaultValue="Choose Frequency"
/>
```
