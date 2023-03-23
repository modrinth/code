import { defineStore } from 'pinia'

export const useTheming = defineStore('theme', {
  state: () => ({ darkTheme: true }),
  actions: {
    toggleTheme() {
      this.darkTheme = !this.darkTheme
    },
  },
})

export const useInstances = defineStore('instances', {
  state: () => ({ instances: [] }),
  actions: {
    fetchInstances() {
      // Fetch from backend.
      const instances = [
        {
          id: 1,
          name: 'Fabulously Optimized',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.18.1',
          downloads: 0,
          trending: false,
          img: 'https://cdn.modrinth.com/data/1KVo5zza/d8152911f8fd5d7e9a8c499fe89045af81fe816e.png',
        },
        {
          id: 2,
          name: 'Create Extra',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.18 ',
          downloads: 0,
          trending: false,
          img: 'https://cdn.modrinth.com/data/xldzprsQ/dd8e20248b82d107f712d69804d0fb9242b29794.png',
        },
        {
          id: 3,
          name: 'All of Fabric: Orion',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.16.5',
          downloads: 0,
          trending: false,
          img: 'https://cdn.modrinth.com/data/mY0lOQFc/81c6eff2b86220e12e62a4ad0d2f364a605c42c4.png',
        },
        {
          id: 4,
          name: 'DeckPack',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 0,
          trending: false,
          img: 'https://cdn.modrinth.com/data/Y6Pi1jae/fccbc9be066ca6d90304a0f0be3a2614806ac4a0.png',
        },
        {
          id: 5,
          name: 'Cabricality',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 1000,
          trending: false,
          img: 'https://cdn.modrinth.com/data/Prk1zzOq/576194b7c614b2a51a6a6f64081288d5f1bef899.png',
        },
        {
          id: 6,
          name: 'Wyncraft 101',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 10000,
          trending: false,
          img: 'https://cdn.modrinth.com/data/yvC0d6Q6/ca38fa0c1f51be61aa7dc83358f79601f5b68dd8.png',
        },
        {
          id: 7,
          name: 'Speedrun Pack',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.7.10',
          downloads: 1000,
          trending: false,
          img: 'https://cdn.modrinth.com/data/1uJaMUOm/a65f66fdbb7e1cf8c4ec6c1c72ff9b029976eeaa.webp',
        },
        {
          id: 8,
          name: 'Wild',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: true,
          img: 'https://cdn.modrinth.com/data/MivxAAtC/icon.jpg',
        },
        {
          id: 9,
          name: 'Cobblemon',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: true,
          img: 'https://cdn.modrinth.com/data/5FFgwNNP/ac7d20b559843f9b8a4013176895603c16c4af86.png',
        },
        {
          id: 10,
          name: 'Old School Minecraft',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: true,
          img: 'https://cdn.modrinth.com/data/whbsULaw/d478287fb35ede9e1b26ca7c3641b4107e67e88d.png',
        },
        {
          id: 11,
          name: 'MCWine',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: true,
          img: 'https://cdn.modrinth.com/data/cXkDMmTw/icon.png',
        },
      ]

      this.instances = [...instances]
    },
  },
})

export const useNews = defineStore('news', {
  state: () => ({ news: [] }),
  actions: {
    fetchNews() {
      // Fetch from backend.
      const news = [
        {
          id: 1,
          headline: 'Caves & Cliffs Update: Part II Dev Q&A',
          blurb: 'Your questions, answered!',
          source: 'From Minecraft.Net',
          img: 'https://cdn.modrinth.com/data/t2q2VlTN/e4d6df0f1a4d6fbbf1b574a264884b3bfe0d28d7.png',
        },
        {
          id: 2,
          headline: 'Project of the WeeK: Test',
          blurb: 'Your questions, answered!',
          source: 'Modrinth Blog',
          img: 'https://cdn.modrinth.com/data/7u6AF3PK/icon.png',
        },
        {
          id: 3,
          headline: 'Minecraft Console Release',
          blurb: 'What did it take?',
          source: 'Modrinth Blog',
          img: 'https://cdn.modrinth.com/data/N6I8tPv6/44a160248230ec8f040cb0aee761df53ae7ebdc6.png',
        },
      ]

      this.news = [...news]
    },
  },
})
