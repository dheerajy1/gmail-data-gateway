import { getActiveGoogleOAuthCreds } from "@/lib/db-scripts/spgetActiveGoogleOAuthCreds";
import { env } from "@/lib/env";
import { google } from "googleapis";

let gmailClient: ReturnType<typeof google.gmail> | null = null;

export async function getGmail() {
  if (gmailClient) return gmailClient;

  const { data } = await getActiveGoogleOAuthCreds({ id: env.Google_APPID });

  const CLIENT_ID = data.clientId;
  const CLIENT_SECRET = data.clientSecret;
  const REDIRECT_URI = "http://localhost:3000";
  const REFRESH_TOKEN = data.refreshToken;

  const oAuth2Client = new google.auth.OAuth2(
    CLIENT_ID,
    CLIENT_SECRET,
    REDIRECT_URI
  );

  oAuth2Client.setCredentials({ refresh_token: REFRESH_TOKEN });

  gmailClient = google.gmail({ version: "v1", auth: oAuth2Client });
  return gmailClient;
}
