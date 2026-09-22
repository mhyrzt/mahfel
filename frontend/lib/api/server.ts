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

export async function createServerApiClient(): Promise<AxiosInstance> {
  const accessToken = await getServerAccessToken();
  if (!accessToken) {
    throw new ServerApiAuthenticationError();
  }
  return axios.create({
    baseURL: serverApiBaseUrl,
    headers: { authorization: `Bearer ${accessToken}` },
    timeout: 10_000,
  });
}
