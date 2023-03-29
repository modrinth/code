# Dropdown
<script setup>
import { ref } from "vue";

const value = ref(null)
</script>

<DemoContainer>
  <DropdownSelect
    id="report-type"
    v-model="value"
    :options="['Daily', 'Weekly', 'Monthly']"
    placeholder="Choose Frequency"
  />
</DemoContainer>

```vue
<DropdownSelect
  id="report-type"
  v-model="reportType"
  :options="['Daily', 'Weekly', 'Monthly']"
  placeholder="Choose Frequency"
/>
```
