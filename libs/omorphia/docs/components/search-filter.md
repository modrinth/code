# Search Filter

<script setup>
import { ref } from "vue"; 

const activeFilters = ref([]);

function toggleFilter(filter) {
  const index = activeFilters.value.indexOf(filter);
  if (index !== -1) {
    activeFilters.value.splice(index, 1)
  } else {
    activeFilters.value.push(filter)
  }
}
</script>

<DemoContainer>
<SearchFilter
  :active-filters="activeFilters"
  display-name="Client"
  facet-name="client"
  @toggle="toggleFilter"
>
  <ClientIcon aria-hidden="true" />
</SearchFilter>
</DemoContainer>

```vue
<script setup>
import { ref } from "vue";

const activeFilters = ref([]);

function toggleFilter(filter) {
  const index = activeFilters.value.indexOf(filter);
  if (index !== -1) {
    activeFilters.value.splice(index, 1)
  } else {
    activeFilters.value.push(filter)
  }
}
</script>

<SearchFilter
  :active-filters="activeFilters"
  display-name="Client"
  facet-name="client"
  @toggle="toggleFilter"
>
  <ClientIcon aria-hidden="true" />
</SearchFilter>
```
