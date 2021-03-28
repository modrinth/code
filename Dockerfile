# Dockerfile
FROM node:14.16.0-alpine

# create destination directory
RUN mkdir -p /usr/src/knossos
WORKDIR /usr/src/knossos

# update and install dependency
RUN apk update && apk upgrade
RUN apk add git

# copy the app, note .dockerignore
COPY . /usr/src/knossos/
RUN npm install
RUN npm run build

EXPOSE 3000

ENV NUXT_HOST=0.0.0.0
ENV NUXT_PORT=3000

ENTRYPOINT [ "npm", "start" ]
