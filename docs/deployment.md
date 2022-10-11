# Deployment

This document guides you through the deployment procedure of `ublog`.

## Prerequisites

You need the following tools during deployment:

- Docker and docker compose.

You also need the following data file during deployment:

- `ublog.db` which holds all your blog's content;
- `site.json` which holds information about your blog site;
- `sslcert.pem` which is your blog site's certificate encoded as a PEM file;
- `sslcert.key` which is the private key of your blog site's certificate encoded as a PEM file.

The following is an example `site.json` file:

```json
{
  "title": "Lancern's Blog",
  "owner": "Lancern",
  "ownerEmail": "msrlancern@gmail.com",
  "url": "https://lancern.xyz",
  "copyright": "Copyright (c) Lancern 2022. All rights reserved.",
  "postUrlTemplate": "https://lancern.xyz/${slug}"
}
```

## Configuration

Before actual deployment, various configuration files needs to be modified.

### Next.js Configuration

Open `ui/next.config.js` and modify the following fields to fit your deployment environment:

- `publicRuntimeConfig.dataServerUrl`, which should be a URL that points to your
  site.

### Nginx Configuration

Open `nginx/nginx.conf` and modify the following fields to fit your deployment environment:

- `http.server.server_name`, which should be the domain name of your site.

### Docker Compose Configuration

Open `docker-compose.yml` and replace all string values in the form `/path/to/xxx` with the real path to the corresponding file or directory.

## Deploy

Execute the following command at the root directory of this repository:

```bashd
docker compose up
```
