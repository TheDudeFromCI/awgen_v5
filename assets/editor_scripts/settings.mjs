import {EventHandler} from "./event_handler.mjs";

/**
 * A small class to manage the project settings.
 */
export class ProjectSettings extends EventHandler {
  #projectName = null;
  #projectVersion = null;
  #tilesets = [];

  /**
   * This constructor initializes the project settings with the given game API.
   * Upon initialization, it will query the engine for the current project
   * settings to cache them.
   * @param {GameAPI} api The game API to use for communication with the engine.
   */
  constructor(api) {
    api.once("engineStarted", (event) => {
      this.#projectName = event.projectName;
      this.#projectVersion = event.projectVersion;

      for (let tileset of event.tilesets) {
        let t = new Tileset(tileset.uuid, tileset.name);
        this.#tilesets.push(t);
      }
    });
  }

  /**
   * Gets the name of the project.
   * @returns {string} The name of the project.
   */
  get name() {
    return this.#projectName;
  }

  /**
   * Gets the version of the project.
   * @returns {string} The version of the project.
   */
  get version() {
    return this.#projectVersion;
  }

  /**
   * Gets the tilesets in the project.
   * @returns {Tileset[]} The tilesets in the project.
   */
  get tilesets() {
    return this.#tilesets;
  }

  /**
   * This function sets the name of the project. Calling this method will update
   * the project settings in the engine.
   * @param {string} name The name of the project.
   */
  async setName(name) {
    let oldName = this.#projectName;
    this.#projectName = name;
    COMMAND({
      command: "setProjectName",
      name: this.#projectName,
    });
    await this.emit("projectNameChanged", this.#projectName, oldName);
  }

  /**
   * This function sets the version of the project. Calling this method will
   * update the project settings in the engine.
   * @param {string} version The version of the project.
   */
  async setVersion(version) {
    let oldVersion = this.#projectVersion;
    this.#projectVersion = version;
    COMMAND({
      command: "setProjectVersion",
      version: this.#projectVersion,
    });
    await this.emit("projectVersionChanged", this.#projectVersion, oldVersion);
  }
}
