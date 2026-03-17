import app from "@/index";
import { isProd } from "@/lib/constants";
import { listenIfPortFree } from "@/lib/port-check"

const port = isProd ? listenIfPortFree({ port: 5000 }) : 5000;

app.listen(port, () => {
  console.log(`🚀 Gmail data gateway api running on http://localhost:${port}`)
});
