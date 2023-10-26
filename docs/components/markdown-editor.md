# Markdown Editor
<script setup>
import { ref } from "vue";

const description = ref(null);
const description2 = ref(null);
const description3 = ref(null);

const onImageUpload = (file) => {
  return URL.createObjectURL(file).replace("blob:", "");
};
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

## With image upload
<DemoContainer>
  <MarkdownEditor v-model="description2" :on-image-upload="onImageUpload" />
</DemoContainer>

```vue
<script setup lang="ts">
import { ref } from "vue";
const description = ref(null)

// Return a URL to the image for the editor to consume
const onImageUpload = (file: File): string => {
  // Upload the file to your server and return a URL
  // This example url will not work bc of proxy
  return URL.createObjectURL(file).replace("blob:", "");
};
</script>

<MarkdownEditor v-model="description" :on-image-upload="onImageUpload" />
```

## Without heading buttons
<DemoContainer>
  <MarkdownEditor v-model="description3" :heading-buttons="false" />
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";
const description = ref(null)
</script>

<MarkdownEditor v-model="description" :heading-buttons="false" />
```
