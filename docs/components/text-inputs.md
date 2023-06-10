# Text Inputs
<script setup>
import { ref } from "vue";
const inputText = ref(null)
</script>

<DemoContainer>
<input
    type="text"
    placeholder="Text input"
/>
</DemoContainer>

```vue
<input
    type="text"
    placeholder="Text input"
/>
```

<DemoContainer>
<div class="iconified-input">
  <SearchIcon/>
  <input
      v-model="inputText"
      type="text"
      placeholder="Text input"
  />
  <Button @click="() => inputText = ''">
    <XIcon/>
  </Button>
</div>
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";
const inputText = ref(null)
</script>

<div class="iconified-input">
  <SearchIcon/>
  <input
    v-model="inputText"
    type="text"
    placeholder="Text input"
  />
  <Button @click="() => inputText = ''">
    <XIcon/>
  </Button>
</div>
```
