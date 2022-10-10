import getConfig from "next/config";
import Image from "next/image";

import AutoFluid from "../components/AutoFluid";
import Navbar from "../components/Navbar";

import "../styles/globals.css";

export default function UblogApp({ Component, pageProps }) {
  const owner = getConfig().publicRuntimeConfig.owner;
  return (
    <>
      <div className="dark:bg-slate-900">
        <Navbar />
        <AutoFluid>
          <div className="mt-28 mb-8">
            <Component {...pageProps} />
          </div>
        </AutoFluid>
      </div>
      <footer className="bg-black pt-8 pb-8">
        <AutoFluid>
          <div className="flex items-center">
            <Image className="rounded-full" src="/avatar-64.jpeg" height="64" width="64" alt="avatar" />
            <div className="text-slate-400 text-sm ml-4">
              <div>Copyright (c) {owner} 2022. All rights reserved.</div>
              <div>Powered by ublog</div>
            </div>
          </div>
        </AutoFluid>
      </footer>
    </>
  );
}
