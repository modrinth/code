import { defineStore } from 'pinia'

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
