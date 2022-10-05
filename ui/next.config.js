/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  publicRuntimeConfig: {
    profile: {
      nickname: "Lancern",
      name: "Sirui Mu",
      area: "Developer / System Security",
      organization: "Tsinghua University",
    },
    uiServerUrl: "https://localhost:3000",
    dataServerUrl: "http://localhost:8000",
  },
}

module.exports = nextConfig
