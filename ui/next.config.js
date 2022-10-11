/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  serverRuntimeConfig: {
    dataServerUrl: "http://ublog-server:30000",
  },
  publicRuntimeConfig: {
    owner: "Lancern",
    dataServerUrl: "https://lancern.xyz",
  },
};

module.exports = nextConfig;
