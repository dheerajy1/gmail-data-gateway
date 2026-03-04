import { google } from "googleapis";
import readline from "readline";
import { env } from "@/lib/env";

const CLIENT_ID = env.Google_CLIENT_ID;
const SCOPE = [
    "https://www.googleapis.com/auth/gmail.readonly", // Gmail
    "https://www.googleapis.com/auth/drive" // Drive access
].join(" ")

const CLIENT_SECRET = env.Google_SECRET_ID;

const REDIRECT_URI = "http://localhost:3000";

const oAuth2Client = new google.auth.OAuth2(
    CLIENT_ID,
    CLIENT_SECRET,
    REDIRECT_URI
);

const authUrl = oAuth2Client.generateAuthUrl({
    access_type: "offline",
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
    rl.close();
});
