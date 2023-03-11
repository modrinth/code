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
          downloads: 10,
          trending: true,
        },
        {
          id: 2,
          name: 'New Caves',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.18 ',
          downloads: 8,
          trending: true,
        },
        {
          id: 3,
          name: 'All the Mods 6',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.16.5',
          downloads: 4,
          trending: true,
        },
        {
          id: 4,
          name: 'Bees',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
        },
        {
          id: 5,
          name: 'SkyFactory 4',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 1000,
          trending: false,
        },
        {
          id: 6,
          name: 'RLCraft',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 10000,
          trending: false,
        },
        {
          id: 7,
          name: 'Regrowth',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.7.10',
          downloads: 1000,
          trending: false,
        },
        {
          id: 8,
          name: 'Birds',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
        },
        {
          id: 9,
          name: 'Dogs',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
        },
        {
          id: 10,
          name: 'Cats',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
        },
        {
          id: 11,
          name: 'Rabbits',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
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
        },
        {
          id: 2,
          headline: 'Project of the WeeK: Gobblygook',
          blurb: 'Your questions, answered!',
          source: 'Modrinth Blog',
        },
        {
          id: 3,
          headline: 'Oreo makes a launcher',
          blurb: 'What did it take?',
          source: 'Modrinth Blog',
        },
      ]

      this.news = [...news]
    },
  },
})
