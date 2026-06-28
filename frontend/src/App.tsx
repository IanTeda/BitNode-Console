import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { router } from "@/router";
import AuthenticationProvider from "@/components/AuthenticationProvider";
import { useAuthentication } from "@/lib/auth-context";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 10,
    },
  },
});

function InnerApp() {
  const auth = useAuthentication();
  return <RouterProvider router={router} context={{ auth }} />;
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthenticationProvider>
        <InnerApp />
      </AuthenticationProvider>
    </QueryClientProvider>
  );
}
