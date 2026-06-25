//-- ./src/components/TanStackDevTools.tsx

import { lazy, Suspense } from "react";

const TanStackRouterDevtools = import.meta.env.DEV
  ? lazy(() =>
      import("@tanstack/react-router-devtools").then((res) => ({
        default: res.TanStackRouterDevtools,
      })),
    )
  : () => null;

export default function TanStackDevTools() {
  return (
    <Suspense>
      <TanStackRouterDevtools position="bottom-right" />
    </Suspense>
  );
}
