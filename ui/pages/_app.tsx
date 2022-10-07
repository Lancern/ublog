import getConfig from "next/config";
import Image from "next/image";

import AutoFluid from "../components/AutoFluid";
import Navbar from "../components/Navbar";

import "../styles/globals.css";

function MyApp({ Component, pageProps }) {
  const profile = getConfig().publicRuntimeConfig.profile;
  return (
    <>
      <div className="dark:bg-slate-900">
        <Navbar />
        <AutoFluid>
          <Component {...pageProps} />
        </AutoFluid>
      </div>
      <footer className="bg-black pt-8 pb-8">
        <AutoFluid>
          <div className="flex items-center">
            <Image className="rounded-full" src="/avatar-64.jpeg" height="64" width="64" />
            <div className="text-slate-400 text-sm ml-4">
              <div>Copyright (c) {profile.nickname} 2022. All rights reserved.</div>
              <div>Powered by ublog</div>
            </div>
          </div>
        </AutoFluid>
      </footer>
    </>
  );
}

export default MyApp;
