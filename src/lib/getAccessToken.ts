import { getActiveGoogleOAuthCreds } from "@/lib/db-scripts/spgetActiveGoogleOAuthCreds";
import { env } from "@/lib/env";
import { google } from "googleapis";

const { data } = await getActiveGoogleOAuthCreds({ id: env.Google_APPID });

const CLIENT_ID = data.clientId;
const CLIENT_SECRET = data.clientSecret;
const REDIRECT_URI = "http://localhost:3000";
const REFRESH_TOKEN = data.refreshToken;

export async function getAccessToken(): Promise<string> {

  const oAuth2Client = new google.auth.OAuth2(
    CLIENT_ID,
    CLIENT_SECRET,
    REDIRECT_URI
  );

  oAuth2Client.setCredentials({
    refresh_token: REFRESH_TOKEN,
  });

  const { token } = await oAuth2Client.getAccessToken();

  if (!token) {
    throw new Error("Failed to generate access token");
  }

  return token;
}
