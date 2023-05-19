# Slider

<script setup>
import { ref } from "vue"; 

const value = ref(0)
const valueTwo = ref(0)
</script>

<DemoContainer>
  <Slider v-model="value" :min="1000" :max="10000" :step="1000"/>
  <Slider v-model="valueTwo" :min="1000" :max="10000" :step="1000" :disabled="true"/>
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";

const value = ref(0)
</script>

<Slider v-model="value" :min="1000" :max="10000" :step="1000"/>
```
