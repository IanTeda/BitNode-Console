import { createRouter } from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import type { AuthContext } from "@/components/AuthenticationProvider";

export const router = createRouter({
  routeTree,
  context: {
    auth: undefined! as AuthContext,
  },
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
