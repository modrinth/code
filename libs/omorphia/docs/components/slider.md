# Slider

<script setup>
import { ref } from "vue"; 

const value = ref(0)
const valueTwo = ref(0)
</script>

<DemoContainer>
  <Slider v-model="value" :min="1000" :max="10000" :step="1000" unit="mb"/>
  <Slider v-model="value" :min="1024" :max="32768" :step="1" :snapPoints='[2048,4096,8192,16384]' :snapRange='500' unit="mb"/>
  <Slider v-model="valueTwo" :min="1000" :max="10000" :step="1000" unit="mb" :disabled="true"/>
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";

const value = ref(0)
</script>

<Slider v-model="value" :min="1000" :max="10000" :step="1000"/>
```
