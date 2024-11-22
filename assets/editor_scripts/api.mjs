import { EventHandler } from "./events.mjs"

export class GameAPI extends EventHandler {
  constructor() {
    super();

    this.on("input", async (message) => {
      print(`Received message: ${message.type}`);

      if (message.type === "engine_started") {
        this.emit("engine_started");
      }

      if (message.type === "project_settings") {
        this.emit("project_settings", [message.name, message.version]);
      }
    });
  }

  async query(event, type) {
    let promise = new Promise((resolve) => {
      this.once(event, resolve);
    });

    NATIVE_SEND({ type });
    return await promise;
  }

  async getProjectSettings() {
    return await this.query("project_settings", "get_project_settings");
  }

  setProjectSettings(name, version) {
    NATIVE_SEND({ type: "set_project_settings", name, version });
  }

  /**
   * This function is the main event loop for the AwgenScript engine.
   * It receives messages from the native layer and forwards them to the event
   * layer of this API. This function will never return.
   */
  async run() {
    while (true) {
      let message = JSON.parse(await NATIVE_QUERY());
      await this.emit('input', message);
    }
  }
}
