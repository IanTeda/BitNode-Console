//-- ./src/components/TanStackDevTools.tsx

import { lazy, Suspense } from "react";

const TanStackRouterDevtools = import.meta.env.DEV
  ? lazy(() =>
      import("@tanstack/react-router-devtools").then((res) => ({
        default: res.TanStackRouterDevtools,
      })),
    )
  : () => null;

const TanStackQueryDevtools = import.meta.env.DEV
  ? lazy(() =>
      import("@tanstack/react-query-devtools").then((res) => ({
        default: res.ReactQueryDevtools,
      })),
    )
  : () => null;

export default function TanStackDevTools() {
  return (
    <Suspense>
      <TanStackRouterDevtools position="bottom-right" />
      <TanStackQueryDevtools buttonPosition="bottom-left" />
    </Suspense>
  );
}
