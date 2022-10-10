FROM node:18-buster

WORKDIR /ublog-ui
COPY . .

RUN npm i
RUN npm run build

ENTRYPOINT ["npm", "run", "start"]
