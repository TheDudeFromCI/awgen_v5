import { EventHandler } from "./events.mjs"

/**
 * This class contains the API for the AwgenScript editor engine. It contains
 * high-level functions that can be used to interact with the engine.
 */
export class GameAPI extends EventHandler {
  /**
   * This constructor initializes the event handler and sets up the event
   * listeners for the engine.
   */
  constructor() {
    super();

    this.on("input", async (message) => {
      print(`Received message: ${message.event}`);

      if (message.event === "engine_started") {
        this.emit("engine_started");
      }

      if (message.event === "query_response") {
        this.emit("query_response", message);
      }
    });
  }

  /**
   * This function sends a query to the native layer of the engine. It will
   * return a promise that resolves when the query is answered by the native
   * layer.
   * @param {string} type The type of query to send to the native layer.
   */
  async #query(type) {
    let promise = new Promise((resolve) => {
      this.once("query_response", resolve);
    });

    COMMAND({ command: "query", query: type });
    return (await promise).data;
  }

  /**
   * This function sends a query to the native layer of the engine to get the
   * project settings. It will return a promise that resolves when the query is
   * answered by the native layer.
   * @returns {Promise<[string, string]>} A promise that resolves with the
   * project settings, as an array with the project name and version.
   */
  async getProjectSettings() {
    return await this.#query("project_settings");
  }

  /**
   * This function sends a command to the native layer of the engine to update
   * the project settings with the given name and version.
   * @param {string} name The name of the project.
   * @param {string} version The version of the project.
   */
  setProjectSettings(name, version) {
    COMMAND({ command: "set_project_settings", name, version });
  }

  /**
   * This async function is the main event loop for the AwgenScript engine.
   * It receives messages from the native layer and forwards them to the event
   * layer of this API. This function will never return, as it is an infinite
   * loop that listens for messages from the native layer.
   */
  async run() {
    while (true) {
      let message = JSON.parse(await EVENT());
      await this.emit('input', message);
    }
  }
}
