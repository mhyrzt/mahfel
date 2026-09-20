const fallbackApiBaseUrl = "https://api.mahfel.localhost";

export const publicApiBaseUrl =
  process.env.NEXT_PUBLIC_MAHFEL_API_URL?.replace(/\/$/, "") ?? fallbackApiBaseUrl;

export const serverApiBaseUrl =
  (process.env.MAHFEL_API_URL ?? process.env.NEXT_SERVER_API_URL ?? publicApiBaseUrl).replace(
    /\/$/,
    "",
  );
