function getRequiredEnv(name) {
  const value = process.env[name];
  if (value === undefined) {
    throw new Error(`Environment variable "${name}" is required but not set.`);
  }

  return value;
}

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  serverRuntimeConfig: {
    dataServerUrl: getRequiredEnv("SSR_DATA_URL"),
  },
  publicRuntimeConfig: {
    owner: getRequiredEnv("SITE_OWNER"),
    dataServerUrl: getRequiredEnv("CSR_DATA_URL"),
  },
};

module.exports = nextConfig;
