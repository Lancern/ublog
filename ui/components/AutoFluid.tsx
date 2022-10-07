import React from "react";

/**
 * A top-level container in the page that keeps whitespace padding areas on both sides on big screens but automatically
 * fills the whole screen width on small screens.
 */
export default function AutoFluid(props: React.PropsWithChildren<{}>): JSX.Element {
  return (
    <div className="lg:flex lg:justify-center lg:px-0 px-2">
      <div className="lg:w-1/2">{props.children}</div>
    </div>
  );
}
