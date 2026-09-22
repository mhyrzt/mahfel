import axios from "axios";

export type ApiFailureKind =
  | "unauthenticated"
  | "forbidden"
  | "validation"
  | "not-found"
  | "conflict"
  | "unavailable"
  | "unexpected";

export interface ApiFailure {
  kind: ApiFailureKind;
  status: number | null;
  code: string | null;
  message: string;
}

interface ErrorEnvelope {
  error: {
    code: string;
    message: string;
  };
}

function isErrorEnvelope(value: unknown): value is ErrorEnvelope {
  if (typeof value !== "object" || value === null || !("error" in value)) {
    return false;
  }
  const { error } = value;
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error &&
    typeof error.code === "string" &&
    typeof error.message === "string"
  );
}

function failureKind(status: number): ApiFailureKind {
  if (status === 401) {
    return "unauthenticated";
  }
  if (status === 403) {
    return "forbidden";
  }
  if (status === 404) {
    return "not-found";
  }
  if (status === 409) {
    return "conflict";
  }
  if (status === 400 || status === 422) {
    return "validation";
  }
  if (status === 429 || status >= 500) {
    return "unavailable";
  }
  return "unexpected";
}

export function apiFailureFromResponse(status: number, data: unknown): ApiFailure {
  const envelope = isErrorEnvelope(data) ? data.error : null;
  return {
    kind: failureKind(status),
    status,
    code: envelope?.code ?? null,
    message: envelope?.message ?? "The request could not be completed.",
  };
}

export function apiFailureFrom(error: unknown): ApiFailure {
  if (!axios.isAxiosError<unknown>(error) || !error.response) {
    return {
      kind: "unavailable",
      status: null,
      code: null,
      message: "The service is unavailable. Please try again.",
    };
  }
  return apiFailureFromResponse(error.response.status, error.response.data);
}
