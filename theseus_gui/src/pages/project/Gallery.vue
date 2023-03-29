<template>
  <div class="gallery">
    <Card v-for="image in gallery" :key="image.url" class="gallery-item">
      <img :src="image.url"  :alt="image.title" class="gallery-image"/>
      <div class="gallery-body">
        <h3>{{ image.title }} </h3>
        {{ image.description }}
      </div>
      <span class="gallery-time">
        <CalendarIcon />
        {{new Date(image.created).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}}
      </span>
    </Card>
  </div>
</template>

<script setup>
import { Card, CalendarIcon } from 'omorphia'
const gallery = [
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/b057daaf50b2a44649022aad767d260725117cd4.png",
    "featured": false,
    "title": "Badlands Creeper",
    "description": "What a nice hat :)",
    "created": "2023-03-03T15:38:11.363404Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/d756f75abf41c80a56d64299f5147d373b4d430d.png",
    "featured": false,
    "title": "Bamboo Creeper",
    "description": "They prefer to hide. But if you plan on making one angry, be sure to bring a panda!",
    "created": "2023-03-03T15:38:38.995796Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/a945a63109b11d0d3769879a6a9828878c57098b.png",
    "featured": false,
    "title": "Beach Creeper",
    "description": "They like to carry their favourite shells on their heads\n\n",
    "created": "2023-03-03T15:39:30.971764Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/137ff566df99319ec7b5289532dac45a4fd98ca2.png",
    "featured": false,
    "title": "Cave Creeper",
    "description": "Just as good at hiding as the bamboo creeper, but much more dangerous :0",
    "created": "2023-03-03T15:40:01.159656Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/61db8a9ec8ffd039ef5af31ae04953c107b6a1a0.png",
    "featured": false,
    "title": "Dark Oak Creeper",
    "description": "Spooky ;-;",
    "created": "2023-03-03T15:40:39.509341Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/2c62df063f68b0e4db19d212abe5199f65275833.png",
    "featured": false,
    "title": "Desert Creeper",
    "description": "More cactus hats!",
    "created": "2023-03-03T15:41:46.434844Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/5a52270c74001bbbfc98b2c504ce100569d7451c.png",
    "featured": false,
    "title": "Dripstone Creeper",
    "description": "Dripped out",
    "created": "2023-03-03T15:42:20.720273Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/4471e9377d224b6cc8e29cfa41db9cb84bacadfd.png",
    "featured": false,
    "title": "Hills Creeper",
    "description": "Admiring the view",
    "created": "2023-03-03T15:42:54.085186Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/7e33e32ade4ad14d941227e6d3d75d5727859c02.png",
    "featured": false,
    "title": "Jungle creeper",
    "description": "Needs a haircut",
    "created": "2023-03-03T15:43:27.998064Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/effc972c752bb61c883bef31e366d17d34342102.png",
    "featured": false,
    "title": "Mushroom Creeper",
    "description": "They are just here to be your friends :)",
    "created": "2023-03-03T15:43:51.228804Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/6c60c62a950f2a8b20dcf7fc2f71bb022445715d.png",
    "featured": false,
    "title": "Ocean Creeper",
    "description": "Just don't touch them and you should be fine",
    "created": "2023-03-03T15:47:12.058613Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/8aa82db07532a30e4744cca4981f96e70dbcb901.png",
    "featured": false,
    "title": "Savannah creeper",
    "description": "Walking totem poles o-o",
    "created": "2023-03-03T15:44:43.161356Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/bc2078d7ea2ae5309d619ae5a67148db79548176.png",
    "featured": false,
    "title": "Snowy Creeper",
    "description": "As long as you don't hurt them, they are pretty chill :) They even help you fight strays!",
    "created": "2023-03-03T15:45:13.367963Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/086fb5d867a9383cf044529f2fce0eec7817735d.png",
    "featured": false,
    "title": "Spruce Creeper",
    "description": "Just little lads :)",
    "created": "2023-03-03T15:45:53.063057Z",
    "ordering": 0
  },
  {
    "url": "https://cdn.modrinth.com/data/MI1LWe93/images/ebd3e997f5159108adc53515f320441692c836f8.png",
    "featured": false,
    "title": "Swamp Creeper",
    "description": "Where did they get the skull ;0;",
    "created": "2023-03-03T15:46:20.690629Z",
    "ordering": 0
  }
]
</script>

<script>
export default {
  name: "Gallery"
}
</script>

<style scoped lang="scss">
.gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
  width: 100%;
  gap: 1rem;
}

.gallery-item {
  padding: 0;
  overflow: hidden;
  margin: 0;
  display: flex;
  flex-direction: column;

  .gallery-image {
    width: 100%;
    aspect-ratio: 2/1;
    object-fit: cover;
    object-position: center;
    cursor: pointer;
  }

  .gallery-body {
    flex-grow: 1;
    padding: 1rem;
  }

  .gallery-time {
    padding: 0 1rem 1rem;
    vertical-align: center;
  }
}
</style>