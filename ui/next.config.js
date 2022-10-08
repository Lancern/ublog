/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  publicRuntimeConfig: {
    owner: "Lancern",
    dataServerUrl: "http://127.0.0.1:8000",
  },
};

module.exports = nextConfig;
