import { EventHandler } from "./events.mjs"
import { ProjectSettings } from "./settings.mjs"

/**
 * This class contains the API for the AwgenScript editor engine. It contains
 * high-level functions that can be used to interact with the engine.
 */
export class EditorAPI extends EventHandler {
  #settings = new ProjectSettings(this);

  /**
   * This function returns the project settings for the engine.
   */
  get settings() {
    return this.#settings;
  }

  /**
   * This function sends a query to the native layer of the engine. It will
   * return a promise that resolves when the query is answered by the native
   * layer.
   * @param {string} type The type of query to send to the native layer.
   * @returns {Promise} A promise that resolves with the response to the query.
   */
  async query(type) {
    let promise = new Promise((resolve) => {
      this.once("query_response", resolve);
    });

    COMMAND({ command: "query", query: type });
    return (await promise).data;
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

      if (message.event === "engine_started") {
        this.emit("engine_started");
      }

      if (message.event === "query_response") {
        this.emit("query_response", message);
      }
    }
  }
}
