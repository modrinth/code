# Notifications

<script setup>
import { ref } from "vue";

const notifsContainer = ref(null);

function addNotification(type) {
  console.log(notifsContainer);
  notifsContainer.value.addNotification({
    title: 'Test Notification',
    text: 'This is a test! Random number: ' + Math.random(),
    type,
  });
}
</script>
<DemoContainer>
  <Notifications ref="notifsContainer" />
  <Button color="primary" @click="addNotification('success')">Success</Button>
  <Button color="highlight" @click="addNotification('warn')">Warning</Button>
  <Button color="danger" @click="addNotification('error')">Error</Button>
  <Button @click="addNotification('info')">Info</Button>
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";

const notifsContainer = ref(null);

function addNotification(type) {
  console.log(notifsContainer);
  notifsContainer.value.addNotification({
    title: 'Test Notification',
    text: 'This is a test! Random number: ' + Math.random(),
    type,
  });
}
</script>

<Notifications ref="notifsContainer" />
<Button color="primary" @click="addNotification('success')">Success</Button>
<Button color="highlight" @click="addNotification('warn')">Warning</Button>
<Button color="danger" @click="addNotification('error')">Error</Button>
<Button @click="addNotification('info')">Info</Button>
```
