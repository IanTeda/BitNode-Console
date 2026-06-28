import { useState, type ComponentProps, type FormEvent } from "react";
import { useNavigate } from "@tanstack/react-router";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import logger from "@/lib/logger";
import { useAuthentication } from "@/components/AuthenticationProvider";
import { AlertCircle, Loader2 } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

const log = logger.getSubLogger({ name: "LoginForm" });

export function LoginAlert({ message }: { message: string }) {
  return (
    <Alert variant="destructive">
      <AlertCircle className="h-4 w-4" />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{message}</AlertDescription>
    </Alert>
  );
}

export function LoginForm({ className, ...props }: ComponentProps<"form">) {
  log.debug("LoginForm rendered");

  const [password, setPassword] = useState("");
  const [errorMessage, setErrorMessage] = useState("");
  const [isPending, setIsPending] = useState(false);
  const navigate = useNavigate();
  const { handleLogin } = useAuthentication();

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setErrorMessage("");
    setIsPending(true);

    try {
      await handleLogin(password);
      await navigate({ to: "/dashboard" });
    } catch (error) {
      const raw = error instanceof Error ? error.message : "";
      const message =
        raw === "Failed to fetch"
          ? "Authentication service is not reachable."
          : decodeURI(raw) || "Login failed. Please try again.";
      setErrorMessage(message);
      log.error("Login error:", message);
    } finally {
      setIsPending(false);
    }
  }

  return (
    <form
      {...props}
      className={cn("flex flex-col gap-6", className)}
      onSubmit={handleSubmit}
    >
      <FieldGroup>
        <div className="flex flex-col items-center gap-1 text-center">
          <h1 className="text-2xl font-bold">Login to BitNode Console</h1>
          <p className="text-sm text-balance text-muted-foreground">
            Enter your console password below to login
          </p>
        </div>
        <FieldError className="text-center">
          {errorMessage && <LoginAlert message={errorMessage} />}
        </FieldError>
        <Field>
          <div className="flex items-center">
            <FieldLabel htmlFor="password">Password</FieldLabel>
          </div>
          <Input
            id="password"
            type="password"
            required
            autoComplete="current-password"
            className="bg-background"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={isPending}
          />
        </Field>
        <Field>
          <Button type="submit" className="w-full" disabled={isPending}>
            {isPending ? (
              <>
                <Loader2 className="animate-spin" />
                Logging in…
              </>
            ) : (
              "Login"
            )}
          </Button>
        </Field>
      </FieldGroup>
    </form>
  );
}
