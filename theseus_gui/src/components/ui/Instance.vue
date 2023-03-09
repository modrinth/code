<script setup>
import { Card, ProjectCard } from 'omorphia'

const props = defineProps({
  display: String,
  instance: Object,
})

const handleInstanceClick = (id) => {
  console.log(id)
}
</script>

<template>
  <div>
    <div
      @click="handleInstanceClick(props.instance.id)"
      class="instance-list-item"
      v-if="display === 'list'"
    >
      <p>{{ props.instance.name }}</p>
    </div>
    <div class="instance-gallery-item" v-else-if="display === 'gallery'">
      <p>{{ props.instance.version }}</p>
      <p>{{ props.instance.name }}</p>
    </div>
    <Card class="instance-card-item" v-else-if="display === 'card'">
      <img src="https://cdn.modrinth.com/data/AANobbMI/icon.png" alt="Trending mod card" />
      <p>{{ props.instance.name }}</p>
    </Card>
    <ProjectCard
      v-else-if="display === 'project'"
      class="instance-project-item"
      type="mod"
      :name="props.instance.name"
      description="Create your own configurable mob spawner!"
      iconUrl="https://cdn-raw.modrinth.com/data/b1LdOZlE/465598dc5d89f67fb8f8de6def21240fa35e3a54.png"
      :downloads="props.instance.downloads.toString()"
      follows="20"
      createdAt="2020-12-27T19:00:00.000Z"
      updatedAt="2021-01-01T19:00:00.000Z"
      :categories="[
        {
          name: 'magic',
          icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><path d=\'M15 4V2\'></path><path d=\'M15 16v-2\'></path><path d=\'M8 9h2\'></path><path d=\'M20 9h2\'></path><path d=\'M17.8 11.8 19 13\'></path><path d=\'M15 9h0\'></path><path d=\'M17.8 6.2 19 5\'></path><path d=\'m3 21 9-9\'></path><path d=\'M12.2 6.2 11 5\'></path></svg>',
        },
        {
          icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><rect x=\'2\' y=\'7\' width=\'20\' height=\'14\' rx=\'2\' ry=\'2\'/><path d=\'M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16\'/></svg>',
          name: 'utility',
        },
        {
          icon: '<svg xmlns=\'http://www.w3.org/2000/svg\' xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'/>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'23\' d=\'m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8\' transform=\'matrix(.08671 0 0 .0867 -49.8 -56)\'/>\n</svg>',
          name: 'fabric',
        },
        {
          icon: '<svg xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-miterlimit=\'1.5\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'></path>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' d=\'M2 7.5h8v-2h12v2s-7 3.4-7 6 3.1 3.1 3.1 3.1l.9 3.9H5l1-4.1s3.8.1 4-2.9c.2-2.7-6.5-.7-8-6Z\'></path>\n</svg>',
          name: 'forge',
        },
      ]"
      projectTypeDisplay="mod"
      projectTypeUrl="/mod"
      serverSide="required"
      clientSide="required"
      :showUpdatedDate="false"
      :color="1716041"
    >
    </ProjectCard>
  </div>
</template>

<style lang="scss" scoped>
.instance-list-item {
  display: inline-block;
  margin: 0.2rem auto;
  cursor: pointer;
  transition: all ease-out 0.1s;

  p {
    font-size: 0.8rem;
  }

  &:hover {
    font-weight: bold;
    filter: brightness(150%);
  }
}

.instance-gallery-item {
  width: 110px;
  height: 110px;
  background: url('https://avatars1.githubusercontent.com/u/6166773?v=4');
  background-position: center;
  background-size: cover;
  margin: 1.3rem;
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  padding: 0.3rem;
  cursor: pointer;
  transition: all ease-in-out 0.2s;
  border-radius: var(--radius-md);

  &:before {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background-image: linear-gradient(to bottom right, rgba(50, 50, 50, 0.9), rgba(0, 0, 0, 0.9));
    opacity: 0.5;
    z-index: 10;
    border-radius: inherit;
  }

  &:hover {
    box-shadow: 0 0 4px 4px rgba(0, 0, 0, 0.5);
  }

  p {
    font-weight: bold;
    font-size: 0.75rem;
    color: #fff;
    z-index: 11;
    margin-bottom: 0.2rem;

    &:nth-child(1) {
      font-weight: normal;
      color: #ddd;
      font-size: 0.6rem;
    }
  }
}

.instance-card-item {
  display: flex;
  justify-content: space-between;
  min-width: 250px;
  padding: 0;
  flex: 1 1 120px;
  cursor: pointer;
  transition: all ease-in-out 0.1s;

  &:hover {
    box-shadow: var(--shadow-raised-lg);
  }

  p {
    font-size: 0.9rem;
    display: inline-block;
    padding: 1rem;
  }

  img {
    width: 50px;
    height: 100%;
    border-radius: var(--radius-md);
  }
}

.instance-project-item {
  width: 100%;
  height: auto;
  margin: 2rem auto;
}
</style>
