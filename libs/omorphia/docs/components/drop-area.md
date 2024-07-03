# Drop Area
<script setup>
import { ref } from "vue";

const files = ref([])
</script>

<DemoContainer>
<DropArea accept="*" @change="files">
    <InfoIcon /> Click to choose a file or drag one onto this page
</DropArea>
</DemoContainer>

```vue
<InfoIcon /> Click to choose a file or drag one onto this page
<DropArea accept="*" />
```
