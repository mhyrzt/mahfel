import axios from "axios";

export const browserApiClient = axios.create({
  baseURL: "/api/backend",
  timeout: 10_000,
});
