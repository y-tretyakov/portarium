#!/usr/bin/env bun
import { createCliRenderer } from "@opentui/core";
import { createRoot } from "@opentui/react";
import App from "../src-tui/App";

const renderer = await createCliRenderer({
  exitOnCtrlC: true,
  targetFps: 30,
});

createRoot(renderer).render(<App />);
