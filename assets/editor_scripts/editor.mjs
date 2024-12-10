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
   * This async function is the main event loop for the AwgenScript engine.
   * It receives messages from the native layer and forwards them to the event
   * layer of this API. This function will never return, as it is an infinite
   * loop that listens for messages from the native layer.
   */
  async run() {
    while (true) {
      let message = JSON.parse(await EVENT());
      switch (message.event) {
        case "engineStarted":
          this.emit("engineStarted");
          break;
      }
    }
  }
}
