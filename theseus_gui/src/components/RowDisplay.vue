<script setup>
import Instance from './ui/Instance.vue'
import News from './ui/News.vue'

const props = defineProps({
  instances: Array,
  news: Array,
  label: String,
})

const shouldRenderInstances = props.instances && props.instances?.length !== 0
const shouldRenderNews = props.news && props.news?.length !== 0
</script>

<template>
  <div class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <div aria-hidden="true"></div>
    </div>
    <section class="mods" v-if="shouldRenderInstances">
      <Instance
        v-for="instance in props.instances"
        :key="instance.id"
        display="gallery"
        :instance="instance"
      />
    </section>
    <section class="news" v-else-if="shouldRenderNews">
      <News v-for="news in props.news" :key="news.id" :news="news" />
    </section>
  </div>
</template>

<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  width: 100%;
  margin-top: 2rem;
  padding: 10px 0;

  &:nth-child(odd) {
    background: rgba(59, 59, 59, 0.5);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: inherit;
    width: 90%;

    p {
      font-size: 16px;
    }

    div {
      background: #fff;
      height: 1px;
      width: 85%;
      border-radius: 5px;
    }
  }

  section {
    display: flex;
    justify-content: space-evenly;
    align-items: inherit;
  }

  .mods {
    margin: auto;
  }

  .news {
    margin: auto;
    width: 100%;
  }
}
</style>
