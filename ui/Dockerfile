FROM node:18-buster

WORKDIR /ublog-ui
COPY . .

RUN npm config set registry http://r.cnpmjs.org
RUN npm i

ENTRYPOINT ["./start.sh"]

HEALTHCHECK --interval=30s --timeout=30s --start-period=120s --retries=3 \
    CMD curl -f http://127.0.0.1:3000 || exit 1
