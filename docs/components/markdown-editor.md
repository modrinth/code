# Markdown Editor
<script setup>
import { ref } from "vue";
const description = ref(null)
const description2 = ref(null)
</script>

The Markdown editor allows for easy formatting of Markdown text whether the user is familiar with Markdown or not. It includes standard shortcuts such as `CTRL+B` for bold, `CTRL+I` for italic, and more. 

## Full editor
<DemoContainer>
  <MarkdownEditor v-model="description" />
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";
const description = ref(null)
</script>

<MarkdownEditor v-model="description" />
```

## Without heading buttons
<DemoContainer>
  <MarkdownEditor v-model="description2" :heading-buttons="false" />
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";
const description = ref(null)
</script>

<MarkdownEditor v-model="description" :heading-buttons="false" />
```
