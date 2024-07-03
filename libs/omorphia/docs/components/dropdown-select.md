# Dropdown
<script setup>
import {ref} from "vue";

const value = ref(null);

const newValue = ref(null);
const options = ref([{ test: 'hello', display: 'no' }, { test: 'nob', display: 'yes' },  { test: 'ball', display: 'eat' }]);
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
    v-model="newValue"
    :options="options"
    placeholder="Choose Frequency"
    :display-name="(name) => name?.display"
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
