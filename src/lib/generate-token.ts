import { getGoogleOAuthApp } from "@/lib/db-scripts/getGoogleOAuthApp";
import { upsertGoogleOAuthToken } from "@/lib/db-scripts/spupsertGoogleOAuthToken";
import { env } from "@/lib/env";
import { isoNowIST } from "@/lib/isoNowIST";
import { google } from "googleapis";
import readline from "readline";

const { data } = await getGoogleOAuthApp({ id: env.Google_APPID });

const CLIENT_ID = data.clientId;
const CLIENT_SECRET = data.clientSecret;
const REDIRECT_URI = "http://localhost:3000";

const SCOPE = [
    "https://www.googleapis.com/auth/gmail.readonly", // Gmail
    "https://www.googleapis.com/auth/drive" // Drive access
].join(" ")

const oAuth2Client = new google.auth.OAuth2(
    CLIENT_ID,
    CLIENT_SECRET,
    REDIRECT_URI
);

const authUrl = oAuth2Client.generateAuthUrl({
    access_type: "offline",
    prompt: "consent",
    scope: SCOPE,
});

console.log("Authorize this app by visiting this URL:\n", authUrl);

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
});

rl.question("Enter the code from that page here: ", async (code) => {
    const { tokens } = await oAuth2Client.getToken(code);

    console.log("\nYour Refresh Token:\n", tokens.refresh_token);

    const { data } = await upsertGoogleOAuthToken({ id: env.Google_APPID, refreshToken: tokens.refresh_token });

    console.log(`${isoNowIST()}\t [genToken:Log]\t Successfully inserted new refresh token for google app id ${data.appId}`);

    rl.close();
});
