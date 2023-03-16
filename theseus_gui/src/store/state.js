import { defineStore } from 'pinia'

export const useTheming = defineStore('themeStore', {
  state: () => ({ darkTheme: true }),
  actions: {
    toggleTheme() {
      this.darkTheme = !this.darkTheme
    },
  },
})

export const useInstances = defineStore('instanceStore', {
  state: () => ({ instances: [], filter: '' }),
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
          img: 'https://cdn.modrinth.com/user/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613.jpeg',
        },
        {
          id: 2,
          name: 'New Caves',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.18 ',
          downloads: 8,
          trending: true,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 3,
          name: 'All the Mods 6',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.16.5',
          downloads: 4,
          trending: true,
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
        {
          id: 4,
          name: 'Bees',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 5,
          name: 'SkyFactory 4',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 1000,
          trending: false,
          img: 'https://cdn.modrinth.com/user/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613.jpeg',
        },
        {
          id: 6,
          name: 'RLCraft',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 10000,
          trending: false,
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
        {
          id: 7,
          name: 'Regrowth',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.7.10',
          downloads: 1000,
          trending: false,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 8,
          name: 'Birds',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://avatars.githubusercontent.com/u/83074853?v=4',
        },
        {
          id: 9,
          name: 'Dogs',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://cdn.modrinth.com/user/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613.jpeg',
        },
        {
          id: 10,
          name: 'Cats',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 11,
          name: 'Rabbits',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
      ]

      this.instances = [...instances]
    },
    setFilter(newFilter) {
      this.filter = newFilter
    },
  },
  getters: {
    getFilteredInstances: (state) => {
      const filteredInstances = state.instances.filter((i) => {
        // When time comes, do more advanced fitlering here
        const normalizedInstanceName = i.name?.toLowerCase()
        if (normalizedInstanceName.includes(state.filter.toLowerCase())) return i
      })

      if (filteredInstances && filteredInstances.length > 0) return filteredInstances

      return state.instances
    },
  },
})

export const useNews = defineStore('newsStore', {
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
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
        {
          id: 2,
          headline: 'Project of the WeeK: Gobblygook',
          blurb: 'Your questions, answered!',
          source: 'Modrinth Blog',
          img: 'https://avatars.githubusercontent.com/t/3923733?s=280&v=4',
        },
        {
          id: 3,
          headline: 'Oreo makes a launcher',
          blurb: 'What did it take?',
          source: 'Modrinth Blog',
          img: 'https://avatars.githubusercontent.com/u/30800863?v=4',
        },
      ]

      this.news = [...news]
    },
  },
})
