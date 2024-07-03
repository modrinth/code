# Checkbox

<script setup>
import { ref } from "vue"; 

const value = ref(false)
</script>

<DemoContainer>
<Checkbox v-model="value">Test</Checkbox>
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";

const value = ref(false)
</script>

<Checkbox v-model="value">Test</Checkbox>
```
