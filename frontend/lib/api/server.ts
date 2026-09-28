import "server-only";

import axios, { type AxiosInstance } from "axios";

import { getServerAccessToken } from "../auth/server";
import { serverApiBaseUrl } from "./config";

export class ServerApiAuthenticationError extends Error {
  constructor() {
    super("An authenticated server session is required.");
    this.name = "ServerApiAuthenticationError";
  }
}

export class ServerApiRequestError extends Error {
  constructor(public readonly status?: number) {
    super(status ? `Server API request failed (${status}).` : "Server API request failed.");
    this.name = "ServerApiRequestError";
  }
}

export async function createServerApiClient(): Promise<AxiosInstance> {
  const accessToken = await getServerAccessToken();
  if (!accessToken) {
    throw new ServerApiAuthenticationError();
  }
  const client = axios.create({
    baseURL: serverApiBaseUrl,
    headers: { authorization: `Bearer ${accessToken}` },
    timeout: 10_000,
  });
  client.interceptors.response.use(undefined, (error: unknown) => {
    const status = axios.isAxiosError(error) ? error.response?.status : undefined;
    if (status === 401) {
      throw new ServerApiAuthenticationError();
    }
    throw new ServerApiRequestError(status);
  });
  return client;
}
