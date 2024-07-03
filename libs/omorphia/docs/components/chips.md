# Chips

<script setup>
import { ref } from "vue"; 

const value = ref('option 1')
</script>
<DemoContainer>
<Chips v-model="value" :items="['option 1', 'option 2', 'option 3', 'option 4']" />
</DemoContainer>

```vue
<script setup>
import { ref } from "vue"; 

const value = ref('option 1')
</script>

<Chips v-model="value" :items="['option 1', 'option 2', 'option 3', 'option 4']" />
```
