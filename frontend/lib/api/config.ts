const fallbackApiBaseUrl = "https://api.mahfel.localhost";

export const publicApiBaseUrl =
  process.env.NEXT_PUBLIC_MAHFEL_API_URL?.replace(/\/$/, "") ?? fallbackApiBaseUrl;
