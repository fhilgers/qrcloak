import { clientOnly } from "@solidjs/start";

const Inner = clientOnly(() => import("~/app-inner"));

export default function App() {
  return <Inner />;
}
