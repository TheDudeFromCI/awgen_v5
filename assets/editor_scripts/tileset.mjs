import { EventHandler } from "./event_handler.js";

/**
 * Class representing a tileset.
 */
export class Tileset extends EventHandler {
  /**
   * Creates a new tileset with the given name and adds it to the engine.
   * @param {string} name
   * @returns
   */
  static async build(name) {
    let uuid = UUID();
    COMMAND({
      type: "editTileset",
      uuid,
      action: {
        action: "create",
        name,
      }
    });
    return new Tileset(uuid, name);
  }

  #uuid = null;
  #name = null;

  /**
   * Creates a new Tileset pointer with the given UUID and name. This
   * constructor should only be used to create a pointer to an existing tileset.
   * To create a completely new tileset, use the `build` method.
   * @param {string} uuid The UUID of the tileset.
   * @param {string} name The name of the tileset.
   */
  constructor(uuid, name) {
    super();
    this.#uuid = uuid;
    this.#name = name;
  }

  /**
   * Gets the UUID of the tileset.
   * @returns {string} The UUID of the tileset.
   */
  get uuid() {
    return this.#uuid;
  }

  /**
   * Gets the name of the tileset.
   * @returns {string} The name of the tileset.
   */
  get name() {
    return this.#name;
  }

  /**
   * Updates the name of the tileset.
   * @param {string} name The new name of the tileset.
   */
  async setName(name) {
    let oldName = this.#name;
    this.#name = name;
    COMMAND({
      type: "editTileset",
      uuid: this.#uuid,
      action: {
        action: "update",
        name: this.#name,
      }
    });
    await this.emit("nameChanged", this.#name, oldName);
  }

  /**
   * Sends a command to the engine to destroy the tileset.
   */
  async destroy() {
    COMMAND({
      type: "editTileset",
      uuid: this.#uuid,
      action: {
        action: "destroy",
      }
    });
  }
}
