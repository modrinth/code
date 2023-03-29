<template>
  <div class="root-container">
    <div class="project-sidebar">
      <Card class="sidebar-card">
        <Avatar size="lg" :src="data.icon_url"/>
        <div class="instance-info">
          <h2 class="name">{{ data.title }}</h2>
          {{ data.description }}
        </div>
        <Categories :categories="categories" :type="type" class="tags">
          <EnvironmentIndicator
              :type-only="moderation"
              :client-side="data.client_side"
              :server-side="data.server_side"
              :type="data.project_type"
          />
        </Categories>
        <hr class="card-divider">
        <div class="button-group">
          <Button color="primary" class="instance-button">
            <DownloadIcon/>
            Install
          </Button>
          <Button class="instance-button" icon-only>
            <ReportIcon/>
          </Button>
          <Button class="instance-button" icon-only>
            <HeartIcon/>
          </Button>
        </div>
        <hr class="card-divider">
        <div class="stats">
          <div class="stat">
            <DownloadIcon aria-hidden="true" />
            <p>
              <strong>{{ data.downloads }}</strong
              ><span class="stat-label"> download<span v-if="data.downloads !== '1'">s</span></span>
            </p>
          </div>
          <div class="stat">
            <HeartIcon aria-hidden="true" />
            <p>
              <strong>{{ data.followers }}</strong
              ><span class="stat-label"> follower<span v-if="data.followers !== '1'">s</span></span>
            </p>
          </div>
          <div class="stat date">
            <EditIcon aria-hidden="true" />
            <span class="date-label">Updated </span> yesterday
          </div>
          <div class="stat date">
            <CalendarIcon aria-hidden="true" />
            <span class="date-label">Published </span>today
          </div>
        </div>
        <hr class="card-divider">
        <div class="links">
          <a
              v-if="data.issues_url"
              :href="data.issues_url"
              class="title"
              rel="noopener nofollow ugc"
          >
            <IssuesIcon aria-hidden="true" />
            <span>Issues</span>
          </a>
          <a
              v-if="data.source_url"
              :href="data.source_url"
              class="title"
              rel="noopener nofollow ugc"
          >
            <CheckIcon aria-hidden="true" />
            <span>Source</span>
          </a>
          <a
              v-if="data.wiki_url"
              :href="data.wiki_url"
              class="title"
              rel="noopener nofollow ugc"
          >
            <WikiIcon aria-hidden="true" />
            <span>Wiki</span>
          </a>
          <a
              v-if="data.wiki_url"
              :href="data.wiki_url"
              class="title"
              rel="noopener nofollow ugc"
          >
            <WikiIcon aria-hidden="true" />
            <span>Discord</span>
          </a>
          <a
              v-for="(donation, index) in data.donation_urls"
              :key="index"
              :href="donation.url"
              rel="noopener nofollow ugc"
          >
            <WikiIcon v-if="donation.id === 'bmac'" aria-hidden="true" />
            <WikiIcon v-else-if="donation.id === 'patreon'" aria-hidden="true" />
            <WikiIcon v-else-if="donation.id === 'ko-fi'" aria-hidden="true" />
            <WikiIcon v-else-if="donation.id === 'paypal'" aria-hidden="true" />
            <WikiIcon
                v-else-if="donation.id === 'open-collective'"
                aria-hidden="true"
            />
            <HeartIcon v-else-if="donation.id === 'github'" />
            <UnknownIcon v-else />
            <span v-if="donation.id === 'bmac'">Buy Me a Coffee</span>
            <span v-else-if="donation.id === 'patreon'">Patreon</span>
            <span v-else-if="donation.id === 'paypal'">PayPal</span>
            <span v-else-if="donation.id === 'ko-fi'">Ko-fi</span>
            <span v-else-if="donation.id === 'github'">GitHub Sponsors</span>
            <span v-else>Donate</span>
          </a>
        </div>
      </Card>
      <Button :link="`modrinth.com/${data.project_type}/${data.slug}`" external>
        <GlobeIcon />
        View in Browser
      </Button>
    </div>
    <div class="content-container">
      <Promotion />
      <Card class="tabs">
        <RouterLink :to="`/project/${this.$route.params.id}/`" class="tab">Description</RouterLink>
        <RouterLink :to="`/project/${this.$route.params.id}/versions`" class="tab">Versions</RouterLink>
        <RouterLink v-if="data.gallery[0]" :to="`/project/${this.$route.params.id}/gallery`" class="tab">Gallery</RouterLink>
      </Card>
      <RouterView/>
    </div>
  </div>
</template>

<script setup>
import {Card, Avatar, Button, DownloadIcon, ReportIcon, HeartIcon, Categories, EnvironmentIndicator, EditIcon, CalendarIcon, IssuesIcon, WikiIcon, CheckIcon, Promotion, GlobeIcon, UnknownIcon} from "omorphia";

const data = {
  "id": "b1LdOZlE",
  "slug": "spirit",
  "project_type": "mod",
  "team": "yAuG205Q",
  "title": "Spirit",
  "description": "Create your own configurable mob spawner!",
  "body": "<span>\n    <img src=\"https://www.bisecthosting.com/images/CF/Spirit/BH_NU_HEADER.png\"\n         alt=\"Spirit banner\" width=\"1100\"/>\n    <center>\n        <h2>Spirit is a magic mod for Forge and Fabric 1.18.2+ that adds new features related to souls and spawning in Minecraft</h2>\n    </center>\n    <a href=\"https://bisecthosting.com/CodexAdrian\">\n        <img src=\"https://www.bisecthosting.com/images/CF/Spirit/BH_NU_PROMO.png\"\n             alt=\"Overview\" width=\"1100\"/>\n    </a>\n    <img src=\"https://cdn.discordapp.com/attachments/881367981463072809/1007044890670137465/BH_NU_BANNER1.png\"\n         alt=\"Overview\" width=\"1100\"/>\n    <center>\n        <h2>\n            Crafting in Spirit revolves around englufing objects in Soul Fire. You can use <a\n                href=\"https://curseforge.com/minecraft/mc-mods/jei\">JEI</a> or <a\n                href=\"https://curseforge.com/minecraft/mc-mods/jei\">REI</a> to view\n            all the recipes in Spirit. If you want some further explanation, this video by SirColor does an excellent job explaining what the main functions of Spirit are\n        </h2>\n    </center>\n    <div>\n        <iframe width=\"900\" height=\"570\" src=\"https://www.youtube.com/embed/lv6C3dZ7hGs\" title=\"YouTube video player\"\n                allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\"\n                allowfullscreen></iframe>\n    </div>\n    <img src=\"https://cdn.discordapp.com/attachments/881367981463072809/1007044890351390720/BH_NU_BANNER2.png\"\n         alt=\"Features\" width=\"1100\"/>\n    <h2>Spirit has a number of features designed to enhance your experience when it comes to soul magic in Minecraft</h2>\n    <blockquote>\n        <h3>Using souls you can:</h3>\n        Summon Monsters<br>\n        Empower Tools<br>\n        Transmute Mobs into other mobs<br>\n        Craft items with soul fire<br>\n    </blockquote>\n    <h2>Detailed descriptions on how you can use these various items, recipes and features can be found on our <a href=\"https://codexadrian.gitbook.io/spirit\">wiki!</a></h2>\n    <a href=\"https://codexadrian.tech/discord\">\n        <img src=\"https://www.bisecthosting.com/images/CF/Spirit/BH_NU_BANNER3.png\"\n             alt=\"Overview\" width=\"1100\"/>\n    </a>\n</span>\n",
  "body_url": null,
  "published": "2022-03-05T23:30:38.854351Z",
  "updated": "2023-03-07T15:32:29.540473Z",
  "approved": "2022-09-17T20:21:07.519053Z",
  "queued": null,
  "status": "approved",
  "requested_status": null,
  "moderator_message": null,
  "license": {
    "id": "LicenseRef-All-Rights-Reserved",
    "name": "",
    "url": null
  },
  "client_side": "required",
  "server_side": "required",
  "downloads": 3628,
  "followers": 25,
  "categories": [
    "magic",
    "utility"
  ],
  "additional_categories": [],
  "game_versions": [
    "1.18.2",
    "1.19.2"
  ],
  "loaders": [
    "fabric",
    "forge"
  ],
  "versions": [
    "PEhhc4aW",
    "guuekcl2",
    "X9sqJmJD",
    "ZkB9qAZW",
    "btBFr95k",
    "4K2VzMbE",
    "FtFcNoht",
    "X7uRgrEW",
    "17lKW7bl",
    "tykm7yBk"
  ],
  "icon_url": "https://cdn.modrinth.com/data/b1LdOZlE/465598dc5d89f67fb8f8de6def21240fa35e3a54.png",
  "issues_url": "https://github.com/terrarium-earth/Spirit/issues",
  "source_url": "https://github.com/terrarium-earth/Spirit",
  "wiki_url": "https://www.curseforge.com/minecraft/mc-mods/spirit",
  "discord_url": "https://discord.gg/Ys8ev2P78d",
  "donation_urls": [
    {
      "id": "ko-fi",
      "platform": "Ko-fi",
      "url": "https://ko-fi.com/codexadrian"
    },
    {
      "id": "ko-fi",
      "platform": "Ko-fi",
      "url": "https://ko-fi.com/codexadrian"
    }
  ],
  "gallery": [
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
  ],
  "flame_anvil_project": null,
  "flame_anvil_user": null,
  "color": 1716041
}

const categories = [
  {
    name: 'magic',
    icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><path d=\'M15 4V2\'></path><path d=\'M15 16v-2\'></path><path d=\'M8 9h2\'></path><path d=\'M20 9h2\'></path><path d=\'M17.8 11.8 19 13\'></path><path d=\'M15 9h0\'></path><path d=\'M17.8 6.2 19 5\'></path><path d=\'m3 21 9-9\'></path><path d=\'M12.2 6.2 11 5\'></path></svg>',
  },
  {
    icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><rect x=\'2\' y=\'7\' width=\'20\' height=\'14\' rx=\'2\' ry=\'2\'/><path d=\'M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16\'/></svg>',
    name: 'utility',
  }
]
</script>

<script>
export default {
  name: "Index"
}
</script>

<style scoped lang="scss">
.root-container {
  display: flex;
  flex-direction: row;
  height: 100%;
}

.project-sidebar {
  width: 20rem;
  min-width: 18rem;
  min-height: 100%;
  background: var(--color-raised-bg);
  padding: 1rem;
}

.sidebar-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  background-color: var(--color-bg);
}

.content-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 1rem;
}

.button-group {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 0.5rem;
}

.stats {
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: var(--gap-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--gap-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      height: var(--stat-strong-size);
      width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }
}

.tabs {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  margin-bottom: var(--gap-md);

  .tab {
    display: flex;
    flex-direction: row;
    align-items: center;
    border-radius: var(--border-radius);
    cursor: pointer;
    transition: background-color 0.2s ease-in-out;

    &:hover {
      background-color: var(--color-raised-bg);
    }

    &.router-view-active {
      background-color: var(--color-raised-bg);
    }
  }
}

.links {
  a {
    display: inline-flex;
    align-items: center;
    border-radius: 1rem;
    color: var(--color-text);

    svg,
    img {
      height: 1rem;
      width: 1rem;
    }

    span {
      margin-left: 0.25rem;
      text-decoration: underline;
      line-height: 2rem;
    }

    &:focus-visible,
    &:hover {
      svg,
      img,
      span {
        color: var(--color-heading);
      }
    }

    &:active {
      svg,
      img,
      span {
        color: var(--color-text-dark);
      }
    }

    &:not(:last-child)::after {
      content: '•';
      margin: 0 0.25rem;
    }
  }
}
</style>