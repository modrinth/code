# Number Inputs

<script setup>
import { ref } from "vue"; 

const input = ref(0)
</script>
<DemoContainer>
<input v-model="input" type="number" />
</DemoContainer>

```vue
<script setup>
import { ref } from "vue"; 

const input = ref(0)
</script>

<input v-model="input" type="number" />
```