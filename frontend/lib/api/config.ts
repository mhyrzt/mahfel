const fallbackApiBaseUrl = "http://localhost:8080";

export const publicApiBaseUrl =
  process.env.NEXT_PUBLIC_MAHFEL_API_URL?.replace(/\/$/, "") ?? fallbackApiBaseUrl;
