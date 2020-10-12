<template>
  <div class="content">
    <h2>Create Mod</h2>
    <input v-model="name" type="text" />
    <input v-model="namespace" type="text" />
    <input v-model="description" type="text" />
    <div class="editor">
      <textarea v-model="body"></textarea>
      <div v-html="$md.render(body)"></div>
    </div>
    <input v-model="issues_url" type="text" />
    <input v-model="source_url" type="text" />
    <input v-model="wiki_url" type="text" />
  </div>
</template>

<script>
import axios from 'axios'

export default {
  data() {
    return {
      name: '',
      namespace: '',
      description: '',
      body: '',
      compiledBody: '',
      versions: [],
      categories: [],
      issues_url: null,
      source_url: null,
      wiki_url: null,
    }
  },
  methods: {
    async createMod() {
      const formData = new FormData()

      formData.append(
        'data',
        JSON.stringify({
          mod_name: this.name,
          mod_namespace: this.namespace,
          mod_description: this.description,
          mod_body: this.body,
          initial_versions: this.versions,
          team_members: [
            {
              user_id: this.$auth.user.id,
              name: this.$auth.user.username,
              role: 'Owner',
            },
          ],
          categories: this.categories,
          issues_url: this.issues_url,
          source_url: this.source_url,
          wiki_url: this.wiki_url,
        })
      )

      try {
        const result = await axios({
          url: 'https://api.modrinth.com/api/v1/mod',
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
          },
        })

        // eslint-disable-next-line no-console
        console.log(result)
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(err)
      }
    },
    compileBody() {
      // this.compiledBody = DomPurify.sanitize(marked(this.body))
    },
  },
}
</script>

<style lang="scss" scoped>
input {
  width: 200px;
  margin-right: auto;
  margin-bottom: 20px;
}

.editor {
  width: 100%;
  min-height: 500px;
  textarea {
    padding: 20px;
    width: calc(50% - 50px);
    min-height: 500px;
    resize: none;
    outline: none;
    border: none;
    margin: 0;
    background-color: var(--color-grey-1);
    border-right: 1px solid var(--color-text);
    color: var(--color-text);
    font-family: monospace;
  }
  div {
    padding: 20px;
    margin: 0;
    min-height: 540px;
    display: inline-block;
    width: 50%;
    box-sizing: border-box;
    vertical-align: top;
    background-color: var(--color-grey-2);
  }
}
</style>
