/**
 * WebdriverIO config driving the REAL compiled Tauri app via the
 * official @wdio/tauri-service in "embedded" mode: the W3C WebDriver
 * server runs INSIDE the app (via the feature-gated
 * tauri-plugin-wdio-webdriver crate) — no external tauri-driver or
 * msedgedriver install needed, on any platform.
 *
 * CI builds the app with `--features wdio` and points TAURI_APP_BINARY_PATH
 * at the binary before running this suite.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const config: Record<string, any> = {
  runner: "local",
  specs: ["./specs/**/*.spec.ts"],
  maxInstances: 1, // one app window at a time
  capabilities: [
    {
      "tauri:options": {
        appBinaryPath:
          process.env.TAURI_APP_BINARY_PATH ??
          "./src-tauri/target/debug/asol-naki.exe",
      },
    },
  ],
  services: [["tauri", { driverProvider: "embedded" }]],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },
  reporters: ["spec"],
  outputDir: "./logs",
  logLevel: "info",
};

export default config;
