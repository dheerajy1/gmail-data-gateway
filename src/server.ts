import app from "@/index";
import { listenIfPortFree } from "@/lib/port-check"
import { env } from "@/lib/env";

const port = listenIfPortFree({ port: env.APP_PORT });

app.listen(port, () => {
  console.log(`🚀 Gmail data gateway api running on http://localhost:${port}`)
});
