import { google } from "googleapis";
import { env } from "@/lib/env";

const CLIENT_ID = env.Google_CLIENT_ID;
const CLIENT_SECRET = env.Google_SECRET_ID;
const REDIRECT_URI = "http://localhost:3000";
const REFRESH_TOKEN = env.Google_REFRESH_TOKEN;

const oAuth2Client = new google.auth.OAuth2(
  CLIENT_ID,
  CLIENT_SECRET,
  REDIRECT_URI
);

oAuth2Client.setCredentials({
  refresh_token: REFRESH_TOKEN,
});

export const gmail = google.gmail({
  version: "v1",
  auth: oAuth2Client,
});
