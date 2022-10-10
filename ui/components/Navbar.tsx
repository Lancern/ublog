import { useState } from "react";
import getConfig from "next/config";
import Image from "next/image";
import Link from "next/link";
import { PropsWithChildren } from "react";

import AutoFluid from "./AutoFluid";
import Toggle from "./Toggle";

export default function Navbar(): JSX.Element {
  const [showMenu, setShowMenu] = useState(false);
  const owner = getConfig().publicRuntimeConfig.owner;

  return (
    <nav className="fixed top-0 left-0 right-0 bg-gray-100/60 dark:bg-gray-800/60 backdrop-blur-sm h-20">
      <AutoFluid>
        <div className="flex justify-center sm:justify-around items-center h-20">
          <Link href="/">
            <div className="flex-grow sm:flex-none flex justify-center sm:justify-start items-center cursor-pointer">
              <Image className="rounded-full" src="/avatar-64.jpeg" width="48" height="48" alt="avatar" />
              <div className="dark:text-white text-lg ml-2 font-bold">
                {owner}
                <span className="ml-1 text-purple-600 dark:text-purple-500 animate-flash">▂</span>
              </div>
            </div>
          </Link>
          <div className="sm:flex-grow hidden sm:flex sm:flex-row-reverse sm:gap-x-8">
            <NavbarLink href="/rss">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth="1.5"
                stroke="currentColor"
                className="inline w-6 h-6 mr-1"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M12.75 19.5v-.75a7.5 7.5 0 00-7.5-7.5H4.5m0-6.75h.75c7.87 0 14.25 6.38 14.25 14.25v.75M6 18.75a.75.75 0 11-1.5 0 .75.75 0 011.5 0z"
                />
              </svg>
              rss
            </NavbarLink>
            <NavbarLink href="/about">about</NavbarLink>
            <NavbarLink href="/posts">articles</NavbarLink>
          </div>
          <div className="flex-none sm:hidden">
            <button
              className="rounded-md border border-gray-400 w-12 h-12 flex justify-center items-center transition focus:bg-gray-300 focus:ring-4 focus:ring-gray-400"
              onClick={() => setShowMenu(!showMenu)}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth="1.5"
                stroke="currentColor"
                className="w-8 h-8"
              >
                <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
              </svg>
            </button>
          </div>
          <Toggle show={showMenu}>
            <div className="fixed right-2 top-16 z-10 rounded-md drop-shadow-md bg-gray-50 dark:bg-slate-700 border border-gray-200 dark:border-slate-600">
              <NavbarLink href="/posts">
                <div className="pl-4 pr-8 py-2">articles</div>
              </NavbarLink>
              <NavbarLink href="/about">
                <div className="pl-4 pr-8 py-2">about</div>
              </NavbarLink>
              <NavbarLink href="/rss">
                <div className="pl-4 pr-8 py-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    strokeWidth="1.5"
                    stroke="currentColor"
                    className="inline w-6 h-6 mr-1"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M12.75 19.5v-.75a7.5 7.5 0 00-7.5-7.5H4.5m0-6.75h.75c7.87 0 14.25 6.38 14.25 14.25v.75M6 18.75a.75.75 0 11-1.5 0 .75.75 0 011.5 0z"
                    />
                  </svg>
                  rss
                </div>
              </NavbarLink>
            </div>
          </Toggle>
        </div>
      </AutoFluid>
    </nav>
  );
}

interface NavbarLinkProps {
  href: string;
}

function NavbarLink(props: PropsWithChildren<NavbarLinkProps>): JSX.Element {
  return (
    <div className="flex items-center">
      <Link href={props.href}>
        <button className="text-gray-700 hover:text-black dark:text-gray-300 dark:hover:text-white">
          {props.children}
        </button>
      </Link>
    </div>
  );
}
