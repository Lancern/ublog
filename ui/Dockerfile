FROM node:18-buster

WORKDIR /ublog-ui
COPY . .

RUN npm config set registry http://r.cnpmjs.org
RUN npm i

ENTRYPOINT ["./start.sh"]
