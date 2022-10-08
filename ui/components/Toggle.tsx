import { PropsWithChildren } from "react";

export interface ToggleProps {
  show: boolean;
}

export default function Toggle({ show, children }: PropsWithChildren<ToggleProps>): JSX.Element {
  if (show) {
    return <>{children}</>;
  } else {
    return <></>;
  }
}
