# Dropdown
<script setup>
import { ref } from "vue";

const value = ref(null)
</script>

<DemoContainer>
  <DropdownSelect
    v-model="value"
    :options="['Daily', 'Weekly', 'Monthly', 'Tomorrow', 'Yesterday', 'Today', 'Biweekly', 'Tuesday', 'January']"
    placeholder="Choose Frequency"
  />
  <DropdownSelect
    v-model="value"
    :options="['Daily', 'Weekly', 'Monthly', 'Tomorrow', 'Yesterday', 'Today', 'Biweekly', 'Tuesday', 'January']"
    placeholder="Choose Frequency"
    render-up
  />
  <DropdownSelect
    v-model="value"
    :options="['Daily', 'Weekly', 'Monthly', 'Tomorrow', 'Yesterday', 'Today', 'Biweekly', 'Tuesday', 'January']"
    placeholder="Choose Frequency"
    disabled
  />
  <DropdownSelect
    v-model="value"
    :options="['Daily', 'Weekly', 'Monthly', 'Tomorrow', 'Yesterday', 'Today', 'Biweekly', 'Tuesday', 'January']"
    placeholder="Choose Frequency"
    :display-name="(name) => name?.toUpperCase()"
  />
</DemoContainer>

```vue
<DropdownSelect
  v-model="value"
  :options="['Daily', 'Weekly', 'Monthly', 'Tomorrow', 'Yesterday', 'Today', 'Biweekly', 'Tuesday', 'January']"
  placeholder="Choose Frequency"
  render-up
/>
```
