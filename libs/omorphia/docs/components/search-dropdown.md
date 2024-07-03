# Search Dropdown

<script setup>
import { ref } from "vue";

const options = ref([]);
const text = ref("");
const onSelected = (option) => {
  options.value.push(option);
};
</script>

<DemoContainer>
{{options}}
{{text}}

<SearchDropdown
  placeholder="Search for dependencies"
  @on-selected="onSelected"
  v-model="text"
  :options="[
    {
      title: 'Project 1',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 2',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 3',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 3',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 1',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 2',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 3',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    },
    {
      title: 'Project 3',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      id: 'ejnfwfinj',
    }
  ]"
/>

</DemoContainer>

```vue
<SearchDropdown
  placeholder="Search for dependencies"
  @on-selected="onSelected"
  v-model="text"
  disabled
  render-up
  circled-icons
  :options="[
    {
      title: 'Project 1',
      subtitle: 'Author 1',
      icon: 'https://cdn.modrinth.com/data/pzd7e1y0/a00d7bc1cec363bb4e678da82f33da5708e8a30a.png',
      ...
    },
    ...
  ]"
/>
```
