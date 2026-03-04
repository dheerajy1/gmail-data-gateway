import jwt from "jsonwebtoken";

export function getTokenExpiry(): Date | null {
    const token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbjEiLCJyb2xlIjoiYWRtaW4iLCJpYXQiOjE3NjcyOTU5OTcsImV4cCI6MTc2NzM4MjM5NywiaXNzIjoicmVjb3JkbG9nLWFwaSJ9.5AezGVFIwFRrN9u_2Q8GKtZ-urm-nt-NQNqP8ot1vcY"

    const decoded = jwt.decode(token) as jwt.JwtPayload | null;
    if (!decoded?.exp) return null;

    console.log(new Date(decoded.exp * 1000))

    return new Date(decoded.exp * 1000);
}
getTokenExpiry()