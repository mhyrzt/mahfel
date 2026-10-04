import { render, screen, fireEvent } from "@testing-library/react";
import { signIn } from "next-auth/react";
import { expect, it, vi } from "vitest";
import { KeycloakSignInButton } from "../components/keycloak-sign-in-button";

vi.mock("next-auth/react", () => ({ signIn: vi.fn() }));

it("starts Keycloak sign-in directly and prevents duplicate clicks", () => {
  vi.mocked(signIn).mockReturnValue(new Promise(() => {}));
  render(<KeycloakSignInButton>Enter Mahfel</KeycloakSignInButton>);
  const button = screen.getByRole("button", { name: "Enter Mahfel" });
  fireEvent.click(button);
  expect(signIn).toHaveBeenCalledWith("keycloak", { callbackUrl: "/workspace" });
  expect(button).toBeDisabled();
});
