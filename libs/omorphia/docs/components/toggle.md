# Toggle

<script setup>
import { ref } from "vue"; 

const value = ref(true)
</script>
<DemoContainer>
  <Toggle v-model="value" checked="true" />
</DemoContainer>

```vue
<script setup>
import { ref } from "vue"; 

const value = ref(true)
</script>

<Toggle v-model="value" />
```
