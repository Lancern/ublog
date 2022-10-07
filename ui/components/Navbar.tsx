import getConfig from "next/config";
import Image from "next/image";
import Link from "next/link";
import { PropsWithChildren } from "react";

import AutoFluid from "./AutoFluid";

export default function Navbar(): JSX.Element {
  const profile = getConfig().publicRuntimeConfig.profile;

  return (
    <nav className="bg-gray-50 dark:bg-gray-800 h-20">
      <AutoFluid>
        <div className="flex items-center h-20">
          <Link href="/">
            <div className="flex-none flex items-center cursor-pointer">
              <Image className="rounded-full" src="/avatar-64.jpeg" width="48" height="48" />
              <div className="dark:text-white text-lg ml-2 font-bold">
                {profile.nickname}
                <span className="ml-1 text-purple-600 dark:text-purple-500 animate-flash">▂</span>
              </div>
            </div>
          </Link>
          <div className="flex-grow flex flex-row-reverse gap-x-8">
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
